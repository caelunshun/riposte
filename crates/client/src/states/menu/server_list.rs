use anyhow::{bail, Context as _};
use duit::{
    widget,
    widgets::{Button, Text},
    WidgetHandle,
};
use riposte_backend_api::{
    client::GameClientToHub, join_game_response, GameList, JoinGameRequest, Uuid,
};
use riposte_common::bridge::{Bridge, ClientSide};

use crate::{
    backend::BackendResponse,
    context::Context,
    generated::ServerListWindow,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

pub enum Action {
    Close,
    JoinGame(Bridge<ClientSide>),
}

struct Close;

struct JoinGame(Uuid);

pub struct ServerListState {
    state: StateAttachment,

    window: ServerListWindow,

    games: BackendResponse<GameList>,
    loaded_games: bool,
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
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Action> {
        if !self.loaded_games {
            if let Some(Ok(res)) = self.games.get() {
                let res = res.get_ref();

                let mut table = self.window.games_table.get_mut();
                table.add_row([
                    ("id", widget(Text::new(text!("Game ID")))),
                    ("join_button", widget(Text::new(text!("Actions")))),
                ]);
                for game in &res.games {
                    let id_text = Text::new(text!(
                        "{}",
                        uuid::Uuid::from(game.game_id.clone().unwrap_or_default()).to_hyphenated()
                    ));

                    let join_button = widget(Button::new());
                    join_button
                        .borrow_mut()
                        .data_mut()
                        .add_child(widget(Text::new(text!("Join"))));
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
            let res: anyhow::Result<Bridge<ClientSide>> = cx.runtime().block_on(async {
                let res = cx
                    .backend()
                    .client()
                    .clone()
                    .join_game(JoinGameRequest {
                        game_id: Some(game_id.into()),
                        auth_token: cx.options().account().auth_token().into(),
                    })
                    .await?;
                match res.into_inner().result.context("missing JoinGame result")? {
                    join_game_response::Result::ErrorMessage(e) => bail!("{}", e),
                    join_game_response::Result::SessionId(session_id) => {
                        let session_id =
                            session_id.try_into().ok().context("malformed session ID")?;
                        let conn = GameClientToHub::connect(session_id).await?;
                        Ok(Bridge::client(conn))
                    }
                }
            });

            match res {
                Ok(bridge) => return Some(Action::JoinGame(bridge)),
                Err(e) => log::error!("Failed to join game: {:?}", e),
            }
        }

        None
    }
}
