use duit::{Rect, Vec2, WindowPositioner};
use glam::vec2;

use crate::{
    context::Context,
    game::{city::City, Game},
    generated::{CityBuildingEntry, CityBuildingsWindow},
    state::StateAttachment,
    tooltips::building::short_building_tooltip,
    ui::Z_FOREGROUND,
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 400.]);

struct Positioner;

impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(
            vec2(0., super::economy::SIZE.y),
            vec2(
                SIZE.x,
                available_space.y - super::economy::SIZE.y - super::culture::SIZE.y,
            ),
        )
    }
}

pub struct BuildingsScreen {
    window: CityBuildingsWindow,
}

impl BuildingsScreen {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<CityBuildingsWindow, _>(Positioner, Z_FOREGROUND);

        Self { window }
    }

    pub fn update_info(&mut self, cx: &Context, _game: &Game, city: &City) {
        let mut entries = self.window.buildings_list.get_mut();
        entries.clear_children();

        for building in city.buildings() {
            let (entry, widget) = cx.ui_mut().create_spec_instance::<CityBuildingEntry>();
            entry
                .building_name
                .get_mut()
                .set_text(text!("{}", building.name));
            entry
                .building_output
                .get_mut()
                .set_text(short_building_tooltip(&building));
            entries.add_child(widget);
        }
    }
}
