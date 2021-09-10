use duit::Event;
use winit::event::MouseButton;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::{event::GameEvent, CityId, Game},
    state::StateAttachment,
    utils::VersionSnapshot,
};

use self::{
    economy::EconomyScreen, info_bar::InfoBar, research::ResearchBar,
    turn_indicator::TurnIndicator, unit_actions::UnitActionBar, unit_info::UnitInfo,
};

mod economy;
mod info_bar;
mod research;
mod turn_indicator;
mod unit_actions;
mod unit_info;

pub enum Action {
    OpenCityScreen(CityId),
}

/// The main in-game user interface.
pub struct MainUi {
    attachment: StateAttachment,

    unit_info: UnitInfo,
    unit_actions: UnitActionBar,
    research_bar: ResearchBar,
    economy_screen: EconomyScreen,
    turn_indicator: TurnIndicator,
    info_bar: InfoBar,

    selected_units_version: VersionSnapshot,
}

impl MainUi {
    pub fn new(cx: &Context, game: &Game) -> Self {
        let attachment = cx.state_manager().create_state();

        let mut unit_info = UnitInfo::new(cx, &attachment);
        let mut unit_actions = UnitActionBar::new(cx, &attachment);
        let mut research_bar = ResearchBar::new(cx, &attachment);
        let mut economy_screen = EconomyScreen::new(cx, &attachment);
        let mut turn_indicator = TurnIndicator::new(cx, &attachment);
        let mut info_bar = InfoBar::new(cx, &attachment);

        unit_info.update_info(cx, game);
        unit_actions.update_info(cx, game);
        research_bar.update_info(game);
        economy_screen.update_info(game);
        turn_indicator.update_info(game);
        info_bar.update_info(cx, game);

        Self {
            attachment,
            unit_info,
            unit_actions,
            research_bar,
            economy_screen,
            turn_indicator,
            info_bar,

            selected_units_version: game.selected_units().version(),
        }
    }

    pub fn update(&mut self, cx: &mut Context, game: &Game, client: &mut Client<GameState>) {
        self.unit_actions.update(cx, game, client);
        self.economy_screen.update(cx, game, client);
        self.turn_indicator.update(game);

        if self.selected_units_version.is_outdated() {
            self.on_selected_units_changed(cx, game);
            self.selected_units_version.update();
        }
    }

    pub fn handle_game_event(&mut self, cx: &mut Context, game: &Game, event: &GameEvent) {
        self.research_bar.handle_game_event(cx, game, event);
        self.economy_screen.handle_game_event(cx, game, event);
        self.turn_indicator.handle_game_event(game, event);
        self.info_bar.handle_game_event(cx, game, event);
        match event {
            GameEvent::UnitUpdated { unit } => {
                // If the unit was selected, we should update the UI
                // to match any changed unit state (e.g., new position or new strength)
                if game.selected_units().contains(*unit) {
                    self.on_selected_units_changed(cx, game);
                }
            }
            _ => {}
        }
    }

    fn on_selected_units_changed(&mut self, cx: &mut Context, game: &Game) {
        self.unit_info.on_selected_units_changed(cx, game);
        self.unit_actions.on_selected_units_changed(cx, game);
    }

    pub fn handle_event(&mut self, _cx: &Context, game: &Game, event: &Event) -> Option<Action> {
        // Check for double-clicked cities
        if let Event::MousePress {
            pos,
            button: MouseButton::Left,
            is_double,
        } = event
        {
            if *is_double {
                let clicked_tile_pos = game.view().tile_pos_for_screen_offset(*pos);
                if let Some(city) = game.city_at_pos(clicked_tile_pos) {
                    return Some(Action::OpenCityScreen(city.id()));
                }
            }
        }
        None
    }
}
