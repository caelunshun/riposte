use duit::{Ui, WindowId};
use glam::vec2;

use crate::{
    game::Game,
    generated::{ErrorPopup, GenesisPopup},
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

    pub fn show_error_popup(&self, ui: &mut Ui, error: &str) {
        let (window, root) = ui.create_spec_instance::<ErrorPopup>();

        window
            .error_text
            .get_mut()
            .set_text(text!("@color[200,30,40][Error: {}]", error));

        let window_id = ui.create_window(root, Center::with_size(vec2(400., 200.)), Z_POPUP);

        window
            .close_button
            .get_mut()
            .on_click(move || ClosePopup(window_id));
    }

    pub fn show_genesis_popup(&self, ui: &mut Ui, game: &Game) {
        let (window, root) = ui.create_spec_instance::<GenesisPopup>();

        window.welcome_text.get_mut().set_text(text!(
        "   The sun rises on 4000 BCE. For eons the {} people have lived a nomadic life. Now they are ready to settle their first city.
        
    {}, lead your people to build a civilization that stands the test of time.", game.the_player().civ().adjective, game.the_player().username()));

        let window_id = ui.create_window(root, Center::with_size(vec2(600., 600.)), Z_POPUP);

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
