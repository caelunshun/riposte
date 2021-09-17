use duit::{
    widget,
    widgets::{Button, Text},
    WidgetHandle,
};
use riposte_backend_api::{GameList, Uuid};

use crate::{
    backend::BackendResponse,
    context::{Context, FutureHandle},
    generated::ServerListWindow,
    server_bridge::ServerBridge,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

pub enum Action {
    Close,
    JoinGame(ServerBridge),
}

struct Close;

struct JoinGame(Uuid);

pub struct ServerListState {
    state: StateAttachment,

    window: ServerListWindow,

    games: BackendResponse<GameList>,
    loaded_games: bool,

    pending_join_game: Option<FutureHandle<anyhow::Result<ServerBridge>>>,
}

impl ServerListState {
    pub fn new(cx: &Context) -> Self {
        let state = cx.state_manager().create_state();

        let (window, _) = state.create_window::<ServerListWindow, _>(FillScreen, Z_FOREGROUND);

        let games = cx.backend().list_games();

        window.back_button.get_mut().on_click(|| Close);

        Self {
            state,
            window,
            games,
            loaded_games: false,
            pending_join_game: None,
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Action> {
        if !self.loaded_games {
            if let Some(Ok(res)) = self.games.get() {
                let res = res.get_ref();

                let mut table = self.window.games_table.get_mut();
                table.add_row([
                    ("id", widget(Text::from_markup("Game ID", vars! {}))),
                    (
                        "join_button",
                        widget(Text::from_markup("Actions", vars! {})),
                    ),
                ]);
                for game in &res.games {
                    let id_text = Text::from_markup(
                        format!(
                            "{}",
                            uuid::Uuid::from(game.game_id.clone().unwrap_or_default())
                                .to_hyphenated()
                        ),
                        vars! {},
                    );

                    let join_button = widget(Button::new());
                    join_button
                        .borrow_mut()
                        .data_mut()
                        .add_child(widget(Text::from_markup("Join", vars! {})));
                    let id = game.game_id.clone().unwrap_or_default();
                    WidgetHandle::<Button>::new(join_button.clone())
                        .get_mut()
                        .on_click(move || JoinGame(id.clone()));

                    table.add_row([("id", widget(id_text)), ("join_button", join_button)]);
                }

                self.loaded_games = true;
            }
        }

        if cx.ui_mut().pop_message::<Close>().is_some() {
            return Some(Action::Close);
        }

        if let Some(JoinGame(game_id)) = cx.ui_mut().pop_message::<JoinGame>() {
            self.pending_join_game = Some(ServerBridge::new_multiplayer(cx, game_id));
        }

        if let Some(pending) = &mut self.pending_join_game {
            if let Some(bridge) = pending.take() {
                match bridge {
                    Ok(bridge) => return Some(Action::JoinGame(bridge)),
                    Err(e) => cx.show_error_popup(&format!("Failed to join game: {}", e)),
                }
                self.pending_join_game = None;
            }
        }

        None
    }
}
