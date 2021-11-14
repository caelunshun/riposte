use std::sync::Arc;

use anyhow::Context;
use duit::Vec2;
use dume::Canvas;
use glam::{uvec2, vec2};
use pollster::block_on;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const WINDOW_TITLE: &str = "Riposte - Beta";
pub const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::Fifo;

pub fn init_graphics_state() -> anyhow::Result<(
    EventLoop<()>,
    Window,
    dume::Context,
    Canvas,
    wgpu::Surface,
    wgpu::Texture,
    Arc<wgpu::Device>,
    Arc<wgpu::Queue>,
)> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(1920 / 2, 1080 / 2))
        .build(&event_loop)
        .context("failed to create window")?;

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .context("failed to find a suitable graphics adapter")?;

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("the_device"),
            features: wgpu::Features::default(),
            limits: wgpu::Limits {
                max_texture_dimension_2d: 16384,
                ..Default::default()
            },
        },
        None,
    ))
    .context("failed to get graphics device")?;

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: dume::TARGET_FORMAT,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PRESENT_MODE,
        },
    );

    let context = dume::Context::builder(Arc::clone(&device), Arc::clone(&queue))
        .glyph_subpixel_steps(uvec2(2, 4)).build();
    context.set_default_font_family("Merriweather");

    let canvas = context.create_canvas(
        logical_size(window.inner_size(), window.scale_factor()),
        window.scale_factor() as f32,
    );

    let sample_texture = create_sample_texture(&device, window.inner_size());

    Ok((
        event_loop,
        window,
        context,
        canvas,
        surface,
        sample_texture,
        device,
        queue,
    ))
}

pub fn create_sample_texture(
    device: &wgpu::Device,
    window_size: PhysicalSize<u32>,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sample_texture"),
        size: wgpu::Extent3d {
            width: window_size.width,
            height: window_size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: dume::SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: dume::TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    })
}

pub fn logical_size(physical: PhysicalSize<u32>, scale: f64) -> Vec2 {
    let logical = physical.to_logical(scale);
    vec2(logical.width, logical.height)
}
