use std::sync::Arc;

use anyhow::Context;
use duit::Vec2;
use dume::Canvas;
use glam::{uvec2, vec2};
use pollster::block_on;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

const WINDOW_TITLE: &str = "Riposte - Beta";
pub const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::Fifo;

pub fn init_graphics_state() -> anyhow::Result<(
    EventLoop<()>,
    Window,
    dume::Context,
    Canvas,
    wgpu::Surface,
    Arc<wgpu::Device>,
    Arc<wgpu::Queue>,
)> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
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
            features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
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
        .glyph_subpixel_steps(uvec2(2, 4))
        .max_mipmap_levels(8)
        .build();
    context.set_default_font_family("Merriweather");

    let canvas = context.create_canvas(
        uvec2(window.inner_size().width, window.inner_size().height),
        window.scale_factor() as f32,
    );

    Ok((event_loop, window, context, canvas, surface, device, queue))
}

pub fn logical_size(physical: PhysicalSize<u32>, scale: f64) -> Vec2 {
    let logical = physical.to_logical(scale);
    vec2(logical.width, logical.height)
}
