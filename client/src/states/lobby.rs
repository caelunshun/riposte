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
use duit::{widget, widgets::Text};

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

        let mut state = Self {
            attachment,

            lobby: GameLobby::new(),
            client,

            window,
        };
        state.recreate_table_slots();
        state
    }

    pub fn update(&mut self, _cx: &mut Context) -> anyhow::Result<()> {
        let events = self
            .client
            .handle_messages(&mut self.lobby)
            .context("failed to handle messages")?;

        for event in events {
            match event {
                client::LobbyEvent::InfoUpdated => self.recreate_table_slots(),
            }
        }

        Ok(())
    }

    fn recreate_table_slots(&mut self) {
        let mut table = self.window.slots_table.get_mut();
        table.clear_rows();

        // Header row
        table.add_row([
            ("name", widget(Text::from_markup("Name", vars! {}))),
            ("status", widget(Text::from_markup("Status", vars! {}))),
        ]);

        for slot in self.lobby.slots() {
            let name = if slot.occupied {
                "Occupied"
            } else {
                "@color{rgb(180, 180, 180)}{<empty>}"
            };
            let status = if !slot.occupied {
                "Open"
            } else if slot.is_ai {
                "AI"
            } else if slot.is_admin {
                "Admin"
            } else {
                "Human"
            };

            table.add_row([
                ("name", widget(Text::from_markup(name, vars! {}))),
                (
                    "status",
                    widget(Text::from_markup("%status", vars! { status => status })),
                ),
            ]);
        }
    }
}
