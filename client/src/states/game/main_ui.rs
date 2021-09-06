use crate::{
    client::{Client, GameState},
    context::Context,
    game::{event::GameEvent, Game},
    state::StateAttachment,
    utils::VersionSnapshot,
};

use self::unit_info::UnitInfo;

mod unit_info;

/// The main in-game user interface.
pub struct MainUi {
    attachment: StateAttachment,
    unit_info: UnitInfo,

    selected_units_version: VersionSnapshot,
}

impl MainUi {
    pub fn new(cx: &Context, game: &Game) -> Self {
        let attachment = cx.state_manager().create_state();

        let unit_info = UnitInfo::new(cx, &attachment);

        Self {
            attachment,
            unit_info,
            selected_units_version: game.selected_units().version(),
        }
    }

    pub fn update(&mut self, cx: &mut Context, game: &Game, _client: &mut Client<GameState>) {
        if self.selected_units_version.is_outdated() {
            self.on_selected_units_changed(cx, game);
            self.selected_units_version.update();
        }
    }

    pub fn handle_game_event(&mut self, cx: &mut Context, game: &Game, event: &GameEvent) {
        if let GameEvent::UnitUpdated { unit } = event {
            // If the unit was selected, we should update the UI
            // to match any changed unit state (e.g., new position or new strength)
            if game.selected_units().contains(*unit) {
                self.on_selected_units_changed(cx, game);
            }
        }
    }

    fn on_selected_units_changed(&mut self, cx: &mut Context, game: &Game) {
        self.unit_info.on_selected_units_changed(cx, game);
    }
}
