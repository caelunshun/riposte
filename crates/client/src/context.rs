use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::OsStr,
    future::Future,
    mem,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use anyhow::Context as _;
use duit::{Spec, Ui, Vec2};
use dume::{Canvas, TextureSetBuilder};
use flume::Receiver;
use glam::{uvec2, vec2};
use once_cell::sync::OnceCell;
use riposte_common::{
    assets::Assets,
    registry::{Building, Civilization, Registry, Resource, Tech, UnitKind},
};
use tokio::runtime::{self, Runtime};
use walkdir::WalkDir;
use winit::{dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop, window::Window};

use crate::{
    asset_loaders::{FontLoader, ImageLoader, JsonLoader, SoundLoader, VideoLoader},
    audio::Audio,
    backend::BackendService,
    options::Options,
    paths::FilePaths,
    popups::PopupWindows,
    saveload::SaveFiles,
    state::StateManager,
    ui::{
        flashing_button::FlashingButton, turn_indicator::TurnIndicatorCircle,
        unit_indicator::UnitIndicator,
    },
};

mod init;

const MIN_TEXTURE_ATLAS_SIZE: u32 = 1024;
const MAX_TEXTURE_ATLAS_SIZE: u32 = 8192;

/// Global state for Riposte.
pub struct Context {
    /// Dume canvas for 2D rendering
    canvas: Rc<RefCell<Canvas>>,

    // Graphics state
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface,

    /// The UI state (duit)
    ui: Rc<RefCell<Ui>>,
    /// Popup window manager
    popup_windows: PopupWindows,

    /// The game window
    window: Window,
    /// Audio player
    audio: Rc<RefCell<Audio>>,
    /// Registry of game files
    registry: Arc<Registry>,
    /// Application state store
    state_manager: StateManager,

    /// The Tokio runtime
    runtime: Runtime,
    /// The connection to the backend service (gRPC)
    backend: BackendService,

    /// Persistent game settings
    options: Rc<RefCell<Options>>,

    /// File path manager
    paths: FilePaths,

    /// Time in seconds since program start
    time: f32,
    /// Program start time
    start: Instant,
    /// Previous frame time
    previous_frame: Instant,
    /// Time elapsed since the previous frame
    dt: f32,

    /// Position of the mouse cursor in logical pixxels
    cursor_pos: Vec2,

    saves: RefCell<SaveFiles>,

    texture_set_builder: Rc<RefCell<Option<TextureSetBuilder>>>,
}

impl Context {
    pub fn new() -> anyhow::Result<(Self, EventLoop<()>)> {
        let (event_loop, window, dume_context, canvas, surface, device, queue) =
            init::init_graphics_state()?;

        let canvas = Rc::new(RefCell::new(canvas));

        let ui = Rc::new(RefCell::new(Ui::new()));
        let popup_windows = PopupWindows::new();

        let state_manager = StateManager::new(Rc::clone(&ui));

        let paths = FilePaths::new()?;

        let options = Options::load_from_disk(&paths)
            .context("failed to load options")?
            .unwrap_or_default();
        let options = Rc::new(RefCell::new(options));

        let audio = Rc::new(RefCell::new(
            Audio::new(Rc::clone(&options)).context("failed to initialize audio player")?,
        ));

        let texture_set_builder = Rc::new(RefCell::new(Some(
            dume_context.create_texture_set_builder(),
        )));

        let mut assets = Assets::new();
        assets
            .add_loader(
                "image",
                ImageLoader::new(&dume_context, Rc::clone(&texture_set_builder)),
            )
            .add_loader("font", FontLoader::new(Rc::clone(&canvas)))
            .add_loader("sound", SoundLoader::new(Rc::clone(&audio)))
            .add_loader("video", VideoLoader::new())
            .add_loader("civ", JsonLoader::<Civilization>::new())
            .add_loader("unit", JsonLoader::<UnitKind>::new())
            .add_loader("tech", JsonLoader::<Tech>::new())
            .add_loader("building", JsonLoader::<Building>::new())
            .add_loader("resource", JsonLoader::<Resource>::new());

        let registry = Arc::new(Registry::new());

        let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
        let rt_handle = runtime.handle().clone();

        let backend = runtime.block_on(async move { BackendService::new(rt_handle).await })?;

        let saves = RefCell::new(SaveFiles::new(&paths));

        assets.load_from_dir("assets")?;

        riposte_common::assets::set_global_assets(assets);

        Ok((
            Self {
                canvas,
                queue,
                device,
                surface,
                ui,
                popup_windows,
                window,
                audio,
                registry,
                state_manager,
                runtime,
                backend,
                options,
                paths,
                start: Instant::now(),
                previous_frame: Instant::now(),
                time: 0.,
                dt: 0.,
                cursor_pos: Vec2::ZERO,
                saves,
                texture_set_builder,
            },
            event_loop,
        ))
    }

    pub fn load_assets(&mut self) -> anyhow::Result<()> {
        Arc::get_mut(&mut self.registry)
            .unwrap()
            .load_from_assets(riposte_common::assets::global_assets());

        let texture_set = self
            .texture_set_builder
            .borrow_mut()
            .take()
            .unwrap()
            .build(MIN_TEXTURE_ATLAS_SIZE, MAX_TEXTURE_ATLAS_SIZE)
            .expect("too many textures - can't fit into texture atlas");
        self.canvas().context().add_texture_set(texture_set);

        Ok(())
    }

    pub fn load_ui_specs(&mut self) -> anyhow::Result<()> {
        self.ui_mut()
            .add_custom_widget("FlashingButton", |_| FlashingButton::new())
            .add_custom_widget("TurnIndicatorCircle", |_| TurnIndicatorCircle::new())
            .add_custom_widget("UnitIndicator", |_| UnitIndicator::new());

        let base_dir = if let Ok(dir) = std::env::var("RIPOSTE_UI_BASE_DIR") {
            PathBuf::from(dir)
        } else {
            PathBuf::from("crates/client/")
        };

        for entry in WalkDir::new(base_dir.join("ui")) {
            let entry = entry?;
            if entry.path().extension() != Some(OsStr::new("yml")) {
                continue;
            }

            let s = fs::read_to_string(entry.path())?;
            self.ui.borrow_mut().add_spec(
                Spec::deserialize_from_str(&s)
                    .with_context(|| format!("malformed spec at '{}'", entry.path().display()))?,
            );
        }

        self.ui_mut()
            .add_stylesheet(&fs::read(base_dir.join("style.yml"))?)
            .context("malformed stylehseet")?;

        Ok(())
    }

    pub fn canvas(&self) -> Ref<Canvas> {
        self.canvas.borrow()
    }

    pub fn canvas_mut(&self) -> RefMut<Canvas> {
        self.canvas.borrow_mut()
    }

    pub fn ui(&self) -> Ref<Ui> {
        self.ui.borrow()
    }

    pub fn ui_mut(&self) -> RefMut<Ui> {
        self.ui.borrow_mut()
    }

    pub fn assets(&self) -> &Assets {
        riposte_common::assets::global_assets()
    }

    pub fn audio(&self) -> Ref<Audio> {
        self.audio.borrow()
    }

    pub fn audio_mut(&self) -> RefMut<Audio> {
        self.audio.borrow_mut()
    }

    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }

    pub fn state_manager(&self) -> &StateManager {
        &self.state_manager
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn options(&self) -> Ref<Options> {
        self.options.borrow()
    }

    pub fn options_mut(&self) -> RefMut<Options> {
        self.options.borrow_mut()
    }

    pub fn paths(&self) -> &FilePaths {
        &self.paths
    }

    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn cursor_pos(&self) -> Vec2 {
        self.cursor_pos
    }

    pub fn saves(&self) -> Ref<SaveFiles> {
        self.saves.borrow()
    }

    pub fn saves_mut(&self) -> RefMut<SaveFiles> {
        self.saves.borrow_mut()
    }

    /// Asynchronously saves the Options to disk.
    pub fn save_options_to_disk(&self) {
        let paths = self.paths.clone();
        let options = self.options.borrow().clone();
        self.spawn_future(async move {
            if let Err(e) = options.save_to_disk(&paths).await {
                log::error!("Failed to save options: {}", e);
            }
        });
    }

    /// Spawns a future to run asynchronously on the Tokio runtime.
    ///
    /// Returns a handle to the future that can be polled each frame.
    pub fn spawn_future<T: Send + 'static>(
        &self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> FutureHandle<T> {
        FutureHandle::spawn(self.runtime.handle(), future)
    }

    pub fn backend(&self) -> &BackendService {
        &self.backend
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: dume::TARGET_FORMAT,
                width: new_size.width,
                height: new_size.height,
                present_mode: init::PRESENT_MODE,
            },
        );
        self.canvas_mut().resize(
            uvec2(new_size.width, new_size.height),
            self.window.scale_factor() as f32,
        );
    }

    pub fn show_error_popup(&mut self, error: &str) {
        self.popup_windows
            .show_error_popup(&mut *self.ui.borrow_mut(), error);
    }

    pub fn update(&mut self) {
        self.dt = self.previous_frame.elapsed().as_secs_f32();
        self.previous_frame = Instant::now();

        self.time = self.start.elapsed().as_secs_f32();

        self.state_manager.update();
        self.audio.borrow_mut().update();
        self.popup_windows.update(&mut *self.ui.borrow_mut());
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            let position = position.to_logical(self.window.scale_factor());
            self.cursor_pos = vec2(position.x, position.y);
        }
    }

    pub fn render(&mut self, render_overlay: impl FnOnce(&mut Self)) {
        let window_logical_size = self
            .window
            .inner_size()
            .to_logical(self.window.scale_factor());
        let window_logical_size = vec2(window_logical_size.width, window_logical_size.height);
        self.ui_mut()
            .render(&mut *self.canvas.borrow_mut(), window_logical_size);

        render_overlay(self);

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => {
                self.surface.configure(
                    &self.device,
                    &wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: dume::TARGET_FORMAT,
                        width: self.window.inner_size().width,
                        height: self.window.inner_size().height,
                        present_mode: init::PRESENT_MODE,
                    },
                );
                self.surface
                    .get_current_texture()
                    .expect("failed to get next swapchain frame")
            }
        };
        self.canvas_mut()
            .render(&frame.texture.create_view(&Default::default()));

        frame.present();
    }

    pub fn popup_windows(&self) -> &PopupWindows {
        &self.popup_windows
    }
}

/// A handle to a future running asynchronously
/// from the main thread.
pub struct FutureHandle<T> {
    receiver: Receiver<T>,
    value: OnceCell<T>,
}

impl<T: Send + 'static> FutureHandle<T> {
    pub fn spawn(
        runtime: &runtime::Handle,
        future: impl Future<Output = T> + Send + 'static,
    ) -> Self {
        let (sender, receiver) = flume::bounded(1);

        runtime.spawn(async move {
            let result = future.await;
            sender.send(result).ok();
        });

        Self {
            receiver,
            value: OnceCell::new(),
        }
    }

    pub fn pending() -> Self {
        Self {
            receiver: flume::bounded(0).1,
            value: OnceCell::new(),
        }
    }

    /// Polls for the return value from the future.
    ///
    /// If the future completed, returns `Some(output)`.
    /// If the future is still running or panicked, returns `None`.
    pub fn get(&self) -> Option<&T> {
        self.value.get_or_try_init(|| self.receiver.try_recv()).ok()
    }

    pub fn take(&mut self) -> Option<T> {
        if self.get().is_some() {
            mem::take(&mut self.value).into_inner()
        } else {
            None
        }
    }
}
