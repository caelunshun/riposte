use std::{
    cell::{Ref, RefCell},
    ptr,
};

use crate::{assets::Handle, backend::BackendResponse, client::{self, Client, LobbyState}, context::Context, generated::GameLobbyWindow, lobby::GameLobby, registry::{Civilization, Leader}, server_bridge::ServerBridge, state::StateAttachment, ui::{FillScreen, Z_FOREGROUND}, utils::color_to_string};

use ahash::{AHashMap, AHashSet};
use duit::{
    widget,
    widgets::{Button, PickList, Text},
};
use protocol::{CreateSlot, DeleteSlot, InitialGameData};
use riposte_backend_api::UserInfo;
use uuid::Uuid;

pub enum Action {
    EnterGame(InitialGameData),
}

enum Message {
    AddAiSlot,
    AddHumanSlot,
    DeleteSlot(u32),
    StartGame,
}

struct SetCiv(Handle<Civilization>);
struct SetLeader(Leader);

/// The game lobby state.
pub struct GameLobbyState {
    attachment: StateAttachment,

    lobby: GameLobby,
    client: Client<client::LobbyState>,

    window: GameLobbyWindow,

    user_info: RefCell<AHashMap<Uuid, BackendResponse<UserInfo>>>,
    missing_user_info: RefCell<AHashSet<Uuid>>,
}

impl GameLobbyState {
    pub fn new_singleplayer(cx: &Context) -> anyhow::Result<Self> {
        let _rt_guard = cx.runtime().enter();
        let bridge = ServerBridge::create_singleplayer(cx.options().account())?;
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

            user_info: RefCell::new(AHashMap::new()),
            missing_user_info: RefCell::new(AHashSet::new()),
        };
        state.recreate_ui(cx);
        state
    }

    pub fn client(&self) -> &Client<LobbyState> {
        &self.client
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<Option<Action>> {
        let events = self
            .client
            .handle_messages(&mut self.lobby, cx.registry())?;

        for event in events {
            match event {
                client::LobbyEvent::InfoUpdated => self.recreate_ui(cx),
                client::LobbyEvent::GameStarted(game_data) => {
                    return Ok(Some(Action::EnterGame(game_data)))
                }
            }
        }

        let mut ui = cx.ui_mut();
        let can_add_slots = self.lobby.slots().len() < cx.registry().num_civs();
        while let Some(msg) = ui.pop_message::<Message>() {
            match msg {
                Message::AddAiSlot => {
                    if can_add_slots {
                        self.client.create_slot(CreateSlot { is_ai: true })
                    }
                }
                Message::AddHumanSlot => {
                    if can_add_slots {
                        self.client.create_slot(CreateSlot { is_ai: false })
                    }
                }
                Message::DeleteSlot(slot_id) => self.client.delete_slot(DeleteSlot { slot_id }),
                Message::StartGame => self.client.request_start_game(),
            }
        }
        while let Some(msg) = ui.pop_message::<SetCiv>() {
            self.client.set_civ_and_leader(&msg.0, &msg.0.leaders[0]);
        }
        while let Some(msg) = ui.pop_message::<SetLeader>() {
            self.client.set_civ_and_leader(
                &self
                    .lobby
                    .player_civ(self.lobby.our_slot().unwrap().id)
                    .unwrap(),
                &msg.0,
            );
        }

        // Check for user info that has been received
        let mut received_user_infos = false;
        for slot in self.lobby.slots() {
            if let Some(owner_uuid) = &slot.owner_uuid {
                let owner_uuid: Uuid = owner_uuid.clone().into();
                if self.user_info.borrow()[&owner_uuid].get().is_some()
                    && self.missing_user_info.borrow().contains(&owner_uuid)
                {
                    self.missing_user_info.borrow_mut().remove(&owner_uuid);
                    received_user_infos = true;
                }
            }
        }
        if received_user_infos {
            self.recreate_ui(cx);
        }

        Ok(None)
    }

    fn user_info(&self, cx: &Context, user: Uuid) -> Option<Ref<UserInfo>> {
        if self.user_info.borrow().contains_key(&user) {
            if self.user_info.borrow()[&user].get().is_some() {
                if self.user_info.borrow()[&user]
                    .get()
                    .as_ref()
                    .unwrap()
                    .is_ok()
                {
                    Some(Ref::map(self.user_info.borrow(), |info| {
                        info[&user].get().unwrap().as_ref().unwrap().get_ref()
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            self.user_info
                .borrow_mut()
                .insert(user, cx.backend().fetch_user_data(user));
            None
        }
    }

    fn recreate_ui(&mut self, cx: &Context) {
        self.recreate_table_slots(cx);

        self.window
            .add_ai_slot_button
            .get_mut()
            .on_click(|| Message::AddAiSlot);
        self.window
            .add_human_slot_button
            .get_mut()
            .on_click(|| Message::AddHumanSlot);

        self.window
            .start_game_button
            .get_mut()
            .on_click(|| Message::StartGame);
    }

    fn recreate_table_slots(&mut self, cx: &Context) {
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
            ("civ", widget(Text::from_markup("Civilization", vars! {}))),
            ("leader", widget(Text::from_markup("Leader", vars! {}))),
        ]);

        for slot in self.lobby.slots() {
            let name = if slot.occupied {
                if let Some(owner_uuid) = &slot.owner_uuid {
                    match self.user_info(cx, owner_uuid.clone().into()) {
                        Some(info) => info.username.clone(),
                        None => {
                            self.missing_user_info
                                .borrow_mut()
                                .insert(owner_uuid.clone().into());
                            "<unknown>".to_owned()
                        }
                    }
                } else {
                    "<AI>".to_owned()
                }
            } else {
                "@color{rgb(180, 180, 180)}{<empty>}".to_owned()
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

            let (civ, leader) = if slot.occupied {
                let civ = self
                    .lobby
                    .player_civ(slot.id)
                    .expect("civ not set for occupied slot");
                (
                    format!("@color{{{}}}{{{}}}", color_to_string(&civ.color), civ.name),
                    slot.leader_name.as_str(),
                )
            } else {
                ("-".to_owned(), "-")
            };

            let (civ_widget, leader_widget) =
                if slot.occupied && ptr::eq(slot, self.lobby.our_slot().unwrap()) {
                    let mut civ_picklist = PickList::new(Some(250.), Some(200.));
                    for civ in cx.registry().civs() {
                        let civ = civ.clone();
                        let option = widget(Text::from_markup(
                            cx.registry().describe_civ(&civ),
                            vars! {},
                        ));
                        civ_picklist.add_option(option, move || SetCiv(civ.clone()));
                    }
                    let civ_picklist = widget(civ_picklist);
                    civ_picklist
                        .borrow_mut()
                        .data_mut()
                        .add_child(widget(Text::from_markup(&civ, vars! {})));

                    let mut leader_picklist = PickList::new(Some(150.), Some(200.));
                    let our_civ = self
                        .lobby
                        .player_civ(self.lobby.our_slot().unwrap().id)
                        .unwrap();
                    for leader in &our_civ.leaders {
                        let leader = leader.clone();
                        let option = widget(Text::from_markup(&leader.name, vars! {}));
                        leader_picklist.add_option(option, move || SetLeader(leader.clone()));
                    }
                    let leader_picklist = widget(leader_picklist);
                    leader_picklist
                        .borrow_mut()
                        .data_mut()
                        .add_child(widget(Text::from_markup(leader, vars! {})));

                    (civ_picklist, leader_picklist)
                } else {
                    (
                        widget(Text::from_markup(&civ, vars! {})),
                        widget(Text::from_markup(leader, vars! {})),
                    )
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
                ("status", widget(Text::from_markup(status, vars! {}))),
                ("civ", civ_widget),
                ("leader", leader_widget),
                ("delete_button", delete_button),
            ]);
        }
    }
}
