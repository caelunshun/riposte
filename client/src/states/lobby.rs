use std::ptr;

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
use duit::{
    widget,
    widgets::{Button, Text},
};
use protocol::{CreateSlot, DeleteSlot};

enum Message {
    AddAiSlot,
    AddHumanSlot,
    DeleteSlot(u32),
}

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
        state.recreate_ui();
        state
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<()> {
        let events = self
            .client
            .handle_messages(&mut self.lobby)
            .context("failed to handle messages")?;

        for event in events {
            match event {
                client::LobbyEvent::InfoUpdated => self.recreate_ui(),
            }
        }

        let mut ui = cx.ui_mut();
        while let Some(msg) = ui.pop_message::<Message>() {
            match msg {
                Message::AddAiSlot => self.client.create_slot(CreateSlot { is_ai: true }),
                Message::AddHumanSlot => self.client.create_slot(CreateSlot { is_ai: false }),
                Message::DeleteSlot(slot_id) => self.client.delete_slot(DeleteSlot { slot_id }),
            }
        }

        Ok(())
    }

    fn recreate_ui(&mut self) {
        self.recreate_table_slots();

        self.window
            .add_ai_slot_button
            .get_mut()
            .on_click(|| Message::AddAiSlot);
        self.window
            .add_human_slot_button
            .get_mut()
            .on_click(|| Message::AddHumanSlot);
    }

    fn recreate_table_slots(&mut self) {
        let mut table = self.window.slots_table.get_mut();
        table.clear_rows();

        // Header row
        table.add_row([
            ("name", widget(Text::from_markup("Name", vars! {}))),
            ("status", widget(Text::from_markup("Status", vars! {}))),
            (
                "delete_button",
                widget(Text::from_markup("Actions", vars! {})),
            ),
        ]);

        for slot in self.lobby.slots() {
            let name = if slot.occupied {
                "Occupied"
            } else {
                "@color{rgb(180, 180, 180)}{<empty>}"
            };
            let status = if !slot.occupied {
                "@color{rgb(30, 200, 50)}{Open}"
            } else if slot.is_ai {
                "@color{rgb(30, 120, 200)}{AI}"
            } else if slot.is_admin {
                "@color{rgb(230, 20, 10)}{Admin}"
            } else {
                "@color{rgb(240, 78, 152)}{Human}"
            };

            let delete_text = if slot.is_ai || !slot.occupied {
                "Remove"
            } else {
                "Kick"
            };

            let mut delete_button = Button::new();
            let id = slot.id;
            delete_button.on_click(move || Message::DeleteSlot(id));
            let delete_button = widget(delete_button);
            delete_button
                .borrow_mut()
                .data_mut()
                .add_child(widget(Text::from_markup(delete_text, vars! {})));

            if ptr::eq(slot, self.lobby.our_slot().unwrap()) {
                delete_button.borrow_mut().data_mut().set_hidden(true);
            }

            table.add_row([
                ("name", widget(Text::from_markup(name, vars! {}))),
                (
                    "status",
                    widget(Text::from_markup(status, vars! {})),
                ),
                ("delete_button", delete_button),
            ]);
        }
    }
}
