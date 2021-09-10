use duit::{Align, Vec2};

use crate::{
    context::Context,
    game::{city::City, Game},
    generated::{CityResourcesEntry, CityResourcesWindow},
    state::StateAttachment,
    tooltips::resource::resource_tooltip,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 400.]);

pub struct ResourcesScreen {
    window: CityResourcesWindow,
}

impl ResourcesScreen {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<CityResourcesWindow, _>(
            AlignFixed::new(SIZE, Align::End, Align::Start),
            Z_FOREGROUND,
        );

        Self { window }
    }

    pub fn update_info(&mut self, cx: &Context, _game: &Game, city: &City) {
        let mut entries = self.window.resources_list.get_mut();
        entries.clear_children();

        for resource in city.resources() {
            let (entry, widget) = cx.ui_mut().create_spec_instance::<CityResourcesEntry>();
            entry
                .resource_name
                .get_mut()
                .set_text(resource.name.clone(), vars! {});
            entry
                .resource_output
                .get_mut()
                .set_text(resource_tooltip(&resource), vars! {});
            entries.add_child(widget);
        }
    }
}
