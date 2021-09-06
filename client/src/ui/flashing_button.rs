use std::{any::Any, f32::consts::TAU, time::Instant};

use glam::Vec2;
use palette::{Mix, Srgba};
use winit::event::MouseButton;

use duit::{
    widget::{Context, LayoutStrategy},
    Color, Event, Widget, WidgetData,
};

pub struct FlashingButton {
    on_click: Option<Box<dyn FnMut() -> Box<dyn Any>>>,
    flashing: bool,
    flashing_start_time: Instant,
}

impl FlashingButton {
    pub fn new() -> Self {
        Self {
            on_click: None,
            flashing: false,
            flashing_start_time: Instant::now(),
        }
    }

    /// Causes a message to be sent when the button is clicked.
    ///
    /// If an `on_click` message is already set, it is overriden.
    pub fn on_click<Message: 'static>(
        &mut self,
        mut message: impl FnMut() -> Message + 'static,
    ) -> &mut Self {
        self.on_click = Some(Box::new(move || Box::new(message())));
        self
    }

    pub fn set_flashing(&mut self, flashing: bool) -> &mut Self {
        self.flashing = flashing;
        self.flashing_start_time = Instant::now();
        self
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    padding: f32,
    border_radius: f32,
    border_width: f32,
    border_color: Color,
    background_color: Color,

    flash_color_a: Color,
    flash_color_b: Color,
    flash_period: f32,
}

impl Widget for FlashingButton {
    type Style = Style;

    fn base_class(&self) -> &str {
        "flashing_button"
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        data.lay_out_child(LayoutStrategy::Shrink, style.padding, &mut cx, max_size);
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        let canvas = &mut cx.canvas;

        let mut background_color: Srgba<u8> = style.background_color.into();
        if self.flashing {
            // Blend colors on top of the background color to create a flashing effect.
            let flash_color_a: Srgba<u8> = style.flash_color_a.into();
            let flash_color_b: Srgba<u8> = style.flash_color_b.into();

            let mix_factor = ((-self.flashing_start_time.elapsed().as_secs_f32() * TAU
                / style.flash_period)
                .cos()
                + 1.)
                / 2.;

            let blend_color = flash_color_a
                .into_format::<f32, f32>()
                .into_linear()
                .mix(&flash_color_b.into_format().into_linear(), mix_factor);

            background_color =
                ((background_color.into_format::<f32, f32>().into_linear() + blend_color) / 2.)
                    .into_encoding()
                    .into_format();
        }

        canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius)
            .solid_color(background_color)
            .fill();
        canvas
            .solid_color(style.border_color.into())
            .stroke_width(style.border_width)
            .stroke();

        data.paint_children(&mut cx);
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        if let Some(on_click) = self.on_click.as_mut() {
            if let Event::MousePress {
                button: MouseButton::Left,
                pos,
            } = event
            {
                if data.bounds().contains(*pos) {
                    cx.send_message((*on_click)());
                }
            }
        }
    }
}
