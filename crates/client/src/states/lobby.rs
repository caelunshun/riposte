use std::{
    cell::{Ref, RefCell},
    ptr,
    sync::Arc,
};

use crate::{
    backend::BackendResponse,
    client::{self, Client, LobbyState},
    context::Context,
    generated::GameLobbyWindow,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

use ahash::{AHashMap, AHashSet};
use anyhow::Context as _;
use duit::{
    widget,
    widgets::{Button, PickList, Text},
};
use palette::Srgba;
use riposte_backend_api::{CreateGameRequest, SessionId, UserInfo};
use riposte_common::protocol::game::server::InitialGameData;
use riposte_common::{
    assets::Handle,
    bridge,
    lobby::{GameLobby, LobbySlot, SlotId, SlotPlayer},
    mapgen::{
        ContinentsSettings, FlatSettings, LandGeneratorSettings, MapSize, MapgenSettings,
        NumContinents,
    },
    protocol::lobby::{CreateSlot, DeleteSlot},
    registry::{Civilization, Leader},
};
use riposte_server::{Server, ServerConfig};
use slotmap::Key;
use strum::IntoEnumIterator;
use uuid::Uuid;

pub enum Action {
    EnterGame(InitialGameData),
}

enum Message {
    AddAiSlot,
    AddHumanSlot,
    DeleteSlot(SlotId),
    StartGame,
    UpdateSettings(Box<dyn FnOnce(&mut MapgenSettings)>),
}

struct SetCiv(Handle<Civilization>);
struct SetLeader(Leader);

/// The game lobby state.
pub struct GameLobbyState {
    attachment: StateAttachment,

    lobby: GameLobby,
    our_slot: SlotId,
    settings: MapgenSettings,

    client: Client<client::LobbyState>,

    window: GameLobbyWindow,

    user_info: RefCell<AHashMap<Uuid, BackendResponse<UserInfo>>>,
    missing_user_info: RefCell<AHashSet<Uuid>>,
}

impl GameLobbyState {
    pub fn new_hosted(
        cx: &Context,
        _save: Option<Vec<u8>>,
        multiplayer: bool,
    ) -> anyhow::Result<Self> {
        cx.runtime().block_on(async move {
            let (server_bridge, client_bridge) = bridge::local_bridge_pair();

            let multiplayer_session_id: Option<SessionId> = if multiplayer {
                let res = cx
                    .backend()
                    .client()
                    .clone()
                    .create_game(CreateGameRequest {
                        auth_token: cx.options().account().auth_token().into(),
                    })
                    .await?;
                Some(res.get_ref().session_id.as_slice().try_into()?)
            } else {
                None
            };

            let mut server = Server::new(ServerConfig {
                registry: Arc::clone(cx.registry()),
                tokio_runtime: cx.runtime().handle().clone(),
                multiplayer_session_id,
            })
            .await?;

            server.add_connection(server_bridge, cx.options().account().uuid(), true);
            cx.runtime().spawn(async move {
                server.run().await;
            });

            let client = Client::new(client_bridge);

            Ok(Self::new(cx, client))
        })
    }

    pub fn new(cx: &Context, client: Client<client::LobbyState>) -> Self {
        let attachment = cx.state_manager().create_state();

        let (window, _) = attachment.create_window::<GameLobbyWindow, _>(FillScreen, Z_FOREGROUND);

        {
            let mut land_type_picklist = window.land_type_picklist.get_mut();
            land_type_picklist.add_option(widget(Text::new(text!("Continents"))), || {
                Message::UpdateSettings(Box::new(|settings| {
                    settings.land = LandGeneratorSettings::Continents(ContinentsSettings {
                        num_continents: NumContinents::Two,
                    })
                }))
            });
            land_type_picklist.add_option(widget(Text::new(text!("Flat"))), || {
                Message::UpdateSettings(Box::new(|settings| {
                    settings.land = LandGeneratorSettings::Flat(FlatSettings { lakes: true })
                }))
            });

            let mut map_size_picklist = window.map_size_picklist.get_mut();
            for map_size in MapSize::iter() {
                map_size_picklist.add_option(
                    widget(Text::new(text!("{:?}", map_size))),
                    move || {
                        Message::UpdateSettings(Box::new(move |settings| {
                            settings.size = map_size;
                        }))
                    },
                );
            }

            let mut num_continents_picklist = window.num_continents_picklist.get_mut();
            for num in NumContinents::iter() {
                num_continents_picklist.add_option(
                    widget(Text::new(text!("{:?}", num))),
                    move || {
                        Message::UpdateSettings(Box::new(move |settings| {
                            if let LandGeneratorSettings::Continents(settings) = &mut settings.land
                            {
                                settings.num_continents = num;
                            }
                        }))
                    },
                );
            }
        }

        let mut state = Self {
            attachment,

            lobby: GameLobby::new(),
            our_slot: SlotId::null(),
            settings: MapgenSettings::default(),
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

    fn our_slot(&self) -> &LobbySlot {
        self.lobby.slot(self.our_slot)
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<Option<Action>> {
        let events = self.client.handle_messages(
            &mut self.lobby,
            &mut self.settings,
            &mut self.our_slot,
            cx.registry(),
        )?;

        for event in events {
            match event {
                client::LobbyEvent::InfoUpdated => self.recreate_ui(cx),
                client::LobbyEvent::GameStarted(game_data) => {
                    return Ok(Some(Action::EnterGame(game_data)))
                }
            }
        }

        let mut ui = cx.ui_mut();
        let can_add_slots = self.lobby.slots().count() < cx.registry().num_civs();
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
                Message::DeleteSlot(slot_id) => self.client.delete_slot(DeleteSlot { id: slot_id }),
                Message::StartGame => self.client.request_start_game(),
                Message::UpdateSettings(f) => {
                    f(&mut self.settings);
                    self.client.set_mapgen_settings(&self.settings);
                }
            }
        }
        while let Some(msg) = ui.pop_message::<SetCiv>() {
            self.client.set_civ_and_leader(&msg.0, &msg.0.leaders[0]);
        }
        while let Some(msg) = ui.pop_message::<SetLeader>() {
            let civ = self
                .our_slot()
                .player
                .civ()
                .context("must be human player")?
                .clone();
            self.client.set_civ_and_leader(&civ, &msg.0);
        }

        // Check for user info that has been received
        let mut received_user_infos = false;
        for (_, slot) in self.lobby.slots() {
            if let SlotPlayer::Human { player_uuid, .. } = &slot.player {
                if self
                    .user_info
                    .borrow()
                    .get(player_uuid)
                    .and_then(|r| r.get())
                    .is_some()
                    && self.missing_user_info.borrow().contains(player_uuid)
                {
                    self.missing_user_info.borrow_mut().remove(player_uuid);
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

        if self.lobby.slots().count() == 0 {
            return;
        }

        let (mut map_size, mut land_type, num_continents) = if self.our_slot().is_admin() {
            (
                self.window.map_size_admin.get_mut(),
                self.window.land_type_admin.get_mut(),
                &self.window.num_continents_admin,
            )
        } else {
            (
                self.window.map_size.get_mut(),
                self.window.land_type.get_mut(),
                &self.window.num_continents,
            )
        };

        map_size.set_text(text!("Map Size: {:?}", self.settings.size));
        land_type.set_text(text!("Land Type: {}", self.settings.land));

        if let LandGeneratorSettings::Continents(settings) = &self.settings.land {
            num_continents
                .get_mut()
                .set_text(text!("# of Continents: {:?}", settings.num_continents));
            self.window.num_continents_picklist.unhide();
        } else {
            self.window.num_continents_picklist.hide();
        };

        if self.our_slot().is_admin() {
            self.window.non_admin_group.hide();
        } else {
            self.window.admin_group.hide();
        }
    }

    fn recreate_table_slots(&mut self, cx: &Context) {
        let mut table = self.window.slots_table.get_mut();
        table.clear_rows();

        // Header row
        table.add_row([
            ("name", widget(Text::new(text!("Name")))),
            ("status", widget(Text::new(text!("Status")))),
            ("delete_button", widget(Text::new(text!("Actions")))),
            ("civ", widget(Text::new(text!("Civilization")))),
            ("leader", widget(Text::new(text!("Leader")))),
        ]);

        for (id, slot) in self.lobby.slots() {
            let name = match &slot.player {
                SlotPlayer::Empty => text!("@color[180, 180, 180][<empty>]"),
                SlotPlayer::Human { player_uuid, .. } => {
                    match self.user_info(cx, player_uuid.clone()) {
                        Some(info) => text!("{}", info.username),
                        None => {
                            self.missing_user_info
                                .borrow_mut()
                                .insert(player_uuid.clone());
                            text!("<unknown>")
                        }
                    }
                }
                SlotPlayer::Ai { .. } => text!("<AI>"),
            };
            let status = match &slot.player {
                SlotPlayer::Empty => text!("@color[30,200,50][Open]"),
                SlotPlayer::Human { is_admin: true, .. } => text!("@color[230,20,10][Admin]"),
                SlotPlayer::Human {
                    is_admin: false, ..
                } => text!("@color[240,78,152][Human]"),
                SlotPlayer::Ai { .. } => text!("@color[30,120,200][AI]"),
            };

            let delete_text = if !matches!(&slot.player, SlotPlayer::Human { .. }) {
                text!("Remove")
            } else {
                text!("Kick")
            };

            let (civ, leader) = if let Some(civ) = slot.player.civ() {
                (
                    text!(
                        "@color[{}][{}]",
                        Srgba::new(civ.color[0], civ.color[1], civ.color[2], 255),
                        civ.name
                    ),
                    text!("{}", slot.player.leader().unwrap().name),
                )
            } else {
                (text!("-"), text!("-"))
            };

            let (civ_widget, leader_widget) = if slot.is_occupied()
                && ptr::eq(slot, self.our_slot())
            {
                let mut civ_picklist = PickList::new(Some(250.), Some(200.));
                for civ in cx.registry().civs() {
                    let civ = civ.clone();
                    let option = widget(Text::new(text!("{}", cx.registry().describe_civ(&civ))));
                    civ_picklist.add_option(option, move || SetCiv(civ.clone()));
                }
                let civ_picklist = widget(civ_picklist);
                civ_picklist
                    .borrow_mut()
                    .data_mut()
                    .add_child(widget(Text::new(civ)));

                let mut leader_picklist = PickList::new(Some(150.), Some(200.));
                let our_civ = self.our_slot().player.civ().unwrap();
                for leader in &our_civ.leaders {
                    let leader = leader.clone();
                    let option = widget(Text::new(text!("{}", leader.name)));
                    leader_picklist.add_option(option, move || SetLeader(leader.clone()));
                }
                let leader_picklist = widget(leader_picklist);
                leader_picklist
                    .borrow_mut()
                    .data_mut()
                    .add_child(widget(Text::new(leader)));

                (civ_picklist, leader_picklist)
            } else {
                (widget(Text::new(civ)), widget(Text::new(leader)))
            };

            let mut delete_button = Button::new();
            delete_button.on_click(move || Message::DeleteSlot(id));
            let delete_button = widget(delete_button);
            delete_button
                .borrow_mut()
                .data_mut()
                .add_child(widget(Text::new(delete_text)));

            if id == self.our_slot {
                delete_button.borrow_mut().data_mut().set_hidden(true);
            }

            table.add_row([
                ("name", widget(Text::new(name))),
                ("status", widget(Text::new(status))),
                ("civ", civ_widget),
                ("leader", leader_widget),
                ("delete_button", delete_button),
            ]);
        }
    }
}
