use crate::{
    client::{self, Client},
    context::Context,
    generated::GameLobbyWindow,
    lobby::GameLobby,
    server_bridge::ServerBridge,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

use anyhow::Context as _;

/// The game lobby state.
pub struct GameLobbyState {
    attachment: StateAttachment,

    lobby: GameLobby,
    client: Client<client::LobbyState>,

    window: GameLobbyWindow,
}

impl GameLobbyState {
    pub fn new_singleplayer(cx: &Context) -> anyhow::Result<Self> {
        let _rt_guard = cx.runtime().enter();
        let bridge = ServerBridge::create_singleplayer()?;
        let client = Client::new(bridge);

        Ok(Self::new(cx, client))
    }

    pub fn new(cx: &Context, client: Client<client::LobbyState>) -> Self {
        let attachment = cx.state_manager().create_state();

        let (window, _) = attachment.create_window::<GameLobbyWindow, _>(FillScreen, Z_FOREGROUND);

        Self {
            attachment,

            lobby: GameLobby::new(),
            client,

            window,
        }
    }

    pub fn update(&mut self, _cx: &mut Context) -> anyhow::Result<()> {
        self.client
            .handle_messages(&mut self.lobby)
            .context("failed to handle messages")?;
        Ok(())
    }
}
