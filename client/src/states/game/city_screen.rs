use duit::Event;
use winit::event::{MouseButton, VirtualKeyCode};

use crate::client::{Client, GameState};
use crate::game::event::GameEvent;
use crate::game::{CityId, Game};
use crate::{context::Context, state::StateAttachment};

use self::{
    buildings::BuildingsScreen, culture::CultureScreen, economy::EconomyScreen,
    info_bar::InfoBarScreen, resources::ResourcesScreen,
};

mod buildings;
mod culture;
mod economy;
mod info_bar;
mod resources;

pub enum Action {
    Close,
}

/// Displays detailed information about a city.
pub struct CityScreen {
    attachment: StateAttachment,

    city: CityId,

    buildings: BuildingsScreen,
    culture: CultureScreen,
    economy: EconomyScreen,
    info_bar: InfoBarScreen,
    resources: ResourcesScreen,
}

impl CityScreen {
    pub fn new(cx: &Context, game: &Game, city: CityId) -> Self {
        let attachment = cx.state_manager().create_state();

        let city = game.city(city);
        let mut view = game.view_mut();
        view.animate_to(cx, city.pos());
        view.animate_zoom_factor_to(cx, 1.);

        let buildings = BuildingsScreen::new(cx, &attachment);
        let culture = CultureScreen::new(cx, &attachment);
        let economy = EconomyScreen::new(cx, &attachment);
        let info_bar = InfoBarScreen::new(cx, &attachment);
        let resources = ResourcesScreen::new(cx, &attachment);

        let mut screen = Self {
            attachment,
            city: city.id(),
            buildings,
            culture,
            economy,
            info_bar,
            resources,
        };
        screen.update_info(cx, game);
        screen
    }

    pub fn handle_game_event(&mut self, cx: &Context, game: &Game, event: &GameEvent) {
        if let GameEvent::CityUpdated { city } = event {
            if *city == self.city {
                self.update_info(cx, game);
            }
        }
    }

    fn update_info(&mut self, cx: &Context, game: &Game) {
        let city = game.city(self.city);
        self.buildings.update_info(cx, game, &city);
        self.culture.update_info(cx, game, &city);
        self.economy.update_info(cx, game, &city);
        self.info_bar.update_info(cx, game, &city);
        self.resources.update_info(cx, game, &city);
    }

    pub fn handle_event(
        &mut self,
        game: &Game,
        client: &mut Client<GameState>,
        event: &Event,
    ) -> Option<Action> {
        // Toggle worked tiles
        if let Event::MousePress {
            pos,
            button: MouseButton::Left,
            ..
        } = event
        {
            let tile = game.view().tile_pos_for_screen_offset(*pos);
            let city = game.city(self.city);
            let worked = !city.is_tile_manually_worked(tile);
            client.set_tile_manually_worked(game, city.id(), tile, worked);
        }

        if let Event::KeyPress {
            key: VirtualKeyCode::Escape,
            ..
        } = event
        {
            Some(Action::Close)
        } else {
            None
        }
    }
}
