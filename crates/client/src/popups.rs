use duit::{Ui, WindowId};
use glam::vec2;

use crate::{
    generated::ErrorPopup,
    ui::{Center, Z_POPUP},
};

struct ClosePopup(WindowId);

/// Popup windows.
#[derive(Default)]
pub struct PopupWindows {}

impl PopupWindows {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show_error_popup(&mut self, ui: &mut Ui, error: &str) {
        let (window, root) = ui.create_spec_instance::<ErrorPopup>();

        window.error_text.get_mut().set_text(
            "@color{rgb(200, 30, 40)}{Error: %error}",
            vars! {
                error => error,
            },
        );

        let window_id = ui.create_window(root, Center::with_size(vec2(400., 200.)), Z_POPUP);

        window
            .close_button
            .get_mut()
            .on_click(move || ClosePopup(window_id));
    }

    pub fn update(&mut self, ui: &mut Ui) {
        loop {
            match ui.pop_message::<ClosePopup>() {
                Some(m) => ui.close_window(m.0),
                None => break,
            }
        }
    }
}
