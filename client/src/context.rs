use std::{
    cell::{Ref, RefCell, RefMut},
    ffi::OsStr,
    future::Future,
    iter,
    rc::Rc,
    sync::Arc,
};

use anyhow::Context as _;
use duit::{Spec, Ui, Vec2};
use dume::Canvas;
use flume::Receiver;
use tokio::runtime::{self, Runtime};
use walkdir::WalkDir;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::Window};

use crate::{
    assets::{
        loaders::{FontLoader, ImageLoader, JsonLoader, SoundLoader},
        Assets,
    },
    audio::Audio,
    backend::BackendService,
    options::Options,
    registry::{Building, Civilization, Registry, Resource, Tech, UnitKind},
    state::StateManager,
};

mod init;

/// Global state for Riposte.
pub struct Context {
    /// Dume canvas for 2D rendering
    canvas: Rc<RefCell<Canvas>>,

    // Graphics state
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface,
    sample_texture: wgpu::Texture,

    /// The UI state (duit)
    ui: Rc<RefCell<Ui>>,
    /// Loaded assets
    assets: Assets,
    /// The game window
    window: Window,
    /// Audio player
    audio: Rc<RefCell<Audio>>,
    /// Registry of game files
    registry: Registry,
    /// Application state store
    state_manager: StateManager,

    /// The Tokio runtime
    runtime: Runtime,
    /// The connection to the backend service (gRPC)
    backend: BackendService,

    /// Persistent game settings
    options: Options,
}

impl Context {
    pub fn new() -> anyhow::Result<(Self, EventLoop<()>)> {
        let (event_loop, window, canvas, surface, sample_texture, device, queue) =
            init::init_graphics_state()?;

        let canvas = Rc::new(RefCell::new(canvas));

        let ui = Rc::new(RefCell::new(Ui::new()));
        let state_manager = StateManager::new(Rc::clone(&ui));

        let audio = Rc::new(RefCell::new(
            Audio::new().context("failed to initialize audio player")?,
        ));

        let mut assets = Assets::new();
        assets
            .add_loader("image", ImageLoader::new(Rc::clone(&canvas)))
            .add_loader("font", FontLoader::new(Rc::clone(&canvas)))
            .add_loader("sound", SoundLoader::new(Rc::clone(&audio)))
            .add_loader("civ", JsonLoader::<Civilization>::new())
            .add_loader("unit", JsonLoader::<UnitKind>::new())
            .add_loader("tech", JsonLoader::<Tech>::new())
            .add_loader("building", JsonLoader::<Building>::new())
            .add_loader("resource", JsonLoader::<Resource>::new());

        let registry = Registry::new();

        let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
        let rt_handle = runtime.handle().clone();

        let backend = runtime.block_on(async move { BackendService::new(rt_handle).await })?;

        let options = Options::default();

        Ok((
            Self {
                canvas,
                queue,
                device,
                surface,
                sample_texture,
                ui,
                assets,
                window,
                audio,
                registry,
                state_manager,
                runtime,
                backend,
                options,
            },
            event_loop,
        ))
    }

    pub fn load_assets(&mut self) -> anyhow::Result<()> {
        self.assets.load_from_dir("assets")?;
        self.registry.load_from_assets(&self.assets);
        Ok(())
    }

    pub fn load_ui_specs(&mut self) -> anyhow::Result<()> {
        for entry in WalkDir::new("/Users/caelum/CLionProjects/riposte/client/ui") {
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
            .add_stylesheet(&fs::read(
                "/Users/caelum/CLionProjects/riposte/client/style.yml",
            )?)
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
        &self.assets
    }

    pub fn audio(&self) -> Ref<Audio> {
        self.audio.borrow()
    }

    pub fn registry(&self) -> &Registry {
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

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.options
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
        self.sample_texture = init::create_sample_texture(&self.device, new_size);
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
    }

    pub fn render(&mut self) {
        self.state_manager.update();

        let window_logical_size = self
            .window
            .inner_size()
            .to_logical(self.window.scale_factor());
        let window_logical_size = Vec2::new(window_logical_size.width, window_logical_size.height);
        self.ui_mut()
            .render(&mut *self.canvas.borrow_mut(), window_logical_size);

        let mut encoder = self.device.create_command_encoder(&Default::default());

        let frame = self
            .surface
            .get_current_frame()
            .expect("failed to get next swapchain frame");
        self.canvas_mut().render(
            &self.sample_texture.create_view(&Default::default()),
            &frame.output.texture.create_view(&Default::default()),
            &mut encoder,
            window_logical_size,
        );

        self.queue.submit(iter::once(encoder.finish()));
    }
}

/// A handle to a future running asynchronously
/// from the main thread.
pub struct FutureHandle<T> {
    receiver: Receiver<T>,
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

        Self { receiver }
    }

    /// Polls for the return value from the future.
    ///
    /// If the future completed, returns `Some(output)`.
    /// If the future is still running or panicked, returns `None`.
    pub fn get(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}
