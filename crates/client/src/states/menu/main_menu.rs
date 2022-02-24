use crate::{
    context::Context,
    generated::{MainMenu, MenuBackground, MenuEntry, UserBar},
    state::StateAttachment,
    ui::{FillScreen, Z_BACKGROUND, Z_FOREGROUND},
};

pub enum Action {
    PushOptions,
    LogOut,
    EnterSingleplayerLobby,
    CreateMultiplayerLobby,
    EnterServerList,
    EnterSavesList { multiplayer: bool },
}

#[derive(Copy, Clone, Debug)]
enum Page {
    Main,
    Singleplayer,
    Multiplayer,
}

#[derive(Debug, Copy, Clone)]

enum Message {
    SingleplayerClicked,
    MultiplayerClicked,
    OptionsClicked,
    BackClicked,
    LogOutClicked,

    SingleplayerLoadGameClicked,
    SingleplayerNewGameClicked,

    MultiplayerJoinGameClicked,
    MultiplayerNewGameClicked,
    MultiplayerLoadGameClicked,
}

pub struct MainMenuState {
    attachment: StateAttachment,
    handle: MainMenu,
    current_page: Page,
}

impl MainMenuState {
    pub fn new(cx: &Context) -> Self {
        let attachment = cx.state_manager().create_state();

        let (handle, _) = attachment.create_window::<UserBar, _>(FillScreen, Z_FOREGROUND);
        handle.user_text.get_mut().set_text(text!(
            "Account: {} - {}",
            cx.options().account().username(),
            cx.options().account().uuid().to_hyphenated()
        ));
        handle
            .log_out_button
            .get_mut()
            .on_click(|| Message::LogOutClicked);

        attachment.create_window::<MenuBackground, _>(FillScreen, Z_BACKGROUND);
        let (handle, _) = attachment.create_window::<MainMenu, _>(FillScreen, Z_FOREGROUND);

        let this = Self {
            handle,
            attachment,
            current_page: Page::Main,
        };

        this.update_entries(cx);

        this
    }

    pub fn update(&mut self, cx: &Context) -> Option<Action> {
        let mut updated = false;
        let mut action = None;
        cx.ui_mut().handle_messages(|msg: &Message| {
            updated = true;
            match msg {
                Message::SingleplayerClicked => self.current_page = Page::Singleplayer,
                Message::BackClicked => self.current_page = Page::Main,
                Message::OptionsClicked => action = Some(Action::PushOptions),
                Message::LogOutClicked => action = Some(Action::LogOut),
                Message::SingleplayerNewGameClicked => {
                    action = Some(Action::EnterSingleplayerLobby)
                }
                Message::SingleplayerLoadGameClicked => {
                    action = Some(Action::EnterSavesList { multiplayer: false })
                }
                Message::MultiplayerClicked => self.current_page = Page::Multiplayer,
                Message::MultiplayerJoinGameClicked => action = Some(Action::EnterServerList),
                Message::MultiplayerNewGameClicked => action = Some(Action::CreateMultiplayerLobby),
                Message::MultiplayerLoadGameClicked => {
                    action = Some(Action::EnterSavesList { multiplayer: true })
                }
            }
        });

        if updated {
            self.update_entries(cx);
        }

        action
    }

    fn update_entries(&self, cx: &Context) {
        self.handle.entries.get_mut().clear_children();

        match self.current_page {
            Page::Main => {
                self.add_entry(cx, "SINGLEPLAYER", Some(Message::SingleplayerClicked))
                    .add_entry(cx, "MULTIPLAYER", Some(Message::MultiplayerClicked))
                    .add_entry(cx, "OPTIONS", Some(Message::OptionsClicked));
            }
            Page::Singleplayer => {
                self.add_entry(cx, "NEW GAME", Some(Message::SingleplayerNewGameClicked))
                    .add_entry(cx, "LOAD GAME", Some(Message::SingleplayerLoadGameClicked))
                    .add_entry(cx, "BACK", Some(Message::BackClicked));
            }
            Page::Multiplayer => {
                self.add_entry(cx, "JOIN GAME", Some(Message::MultiplayerJoinGameClicked))
                    .add_entry(cx, "CREATE GAME", Some(Message::MultiplayerNewGameClicked))
                    .add_entry(cx, "LOAD GAME", Some(Message::MultiplayerLoadGameClicked))
                    .add_entry(cx, "BACK", Some(Message::BackClicked));
            }
        }
    }

    fn add_entry(&self, cx: &Context, name: &str, message: Option<Message>) -> &Self {
        let (handle, entry) = cx.ui_mut().create_spec_instance::<MenuEntry>();
        handle.the_text.get_mut().set_text(text!("{}", name));

        if let Some(msg) = message {
            handle.clickable.get_mut().on_click(move || msg.clone());
        }

        self.handle.entries.get_mut().add_child(entry);
        self
    }
}
