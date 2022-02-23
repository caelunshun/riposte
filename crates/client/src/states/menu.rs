use crate::{
    audio::{SoundCategory, SoundHandle},
    context::Context,
    generated::MenuBackground,
    options::Account,
    state::StateAttachment,
    ui::{FillScreen, Z_BACKGROUND},
    volumes,
};

use self::{
    login::LoginState, main_menu::MainMenuState, options::OptionsState, saves_list::SavesListState,
    server_list::ServerListState,
};

mod login;
mod main_menu;
pub mod options;
mod saves_list;
mod server_list;

enum State {
    MainMenu(MainMenuState),
    Login(LoginState),
    Options(OptionsState),
    ServerList(ServerListState),
    SavesList(SavesListState),
}

pub struct MenuState {
    state: State,
    attachment: StateAttachment,
    music: SoundHandle,
}

impl MenuState {
    pub fn new(cx: &Context) -> Self {
        let attachment = cx.state_manager().create_state();

        attachment.create_window::<MenuBackground, _>(FillScreen, Z_BACKGROUND);

        let music =
            cx.audio()
                .play_looping("music/menu", SoundCategory::Music, volumes::MENU_MUSIC);

        let state = if cx.options().has_account() {
            State::MainMenu(MainMenuState::new(cx))
        } else {
            State::Login(LoginState::new(cx))
        };

        Self {
            state,
            attachment,
            music,
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<crate::Action> {
        let mut action = None;
        match &mut self.state {
            State::MainMenu(s) => match s.update(cx) {
                Some(act) => match act {
                    main_menu::Action::PushOptions => {
                        self.state = State::Options(OptionsState::new(cx))
                    }
                    main_menu::Action::LogOut => {
                        cx.options_mut().clear_account();
                        cx.save_options_to_disk();
                        self.state = State::Login(LoginState::new(cx));
                    }
                    main_menu::Action::EnterSingleplayerLobby => {
                        action = Some(crate::Action::EnterSingleplayerLobby(None))
                    }
                    main_menu::Action::EnterServerList => {
                        self.state = State::ServerList(ServerListState::new(cx))
                    }
                    main_menu::Action::EnterSavesList => {
                        self.state = State::SavesList(SavesListState::new(cx));
                    }
                    main_menu::Action::CreateMultiplayerLobby => {
                        action = Some(crate::Action::EnterMultiplayerLobby)
                    }
                },
                None => {}
            },
            State::Login(s) => {
                let authenticated = s.update(cx);
                if let Some(authenticated) = authenticated {
                    cx.options_mut()
                        .set_account(Account::from_authentication(authenticated));
                    cx.save_options_to_disk();
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
            }
            State::Options(s) => {
                if let Some(options::Action::Close) = s.update(cx) {
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
            }
            State::ServerList(s) => match s.update(cx) {
                Some(server_list::Action::Close) => {
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
                Some(server_list::Action::JoinGame(bridge)) => {
                    action = Some(crate::Action::EnterLobby(bridge));
                }
                _ => {}
            },
            State::SavesList(s) => match s.update(cx) {
                Some(saves_list::Action::Close) => {
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
                Some(saves_list::Action::LoadGame(save)) => {
                    action = Some(crate::Action::EnterSingleplayerLobby(Some(save)));
                }
                _ => {}
            },
        }
        action
    }

    pub fn render_overlay(&mut self, _cx: &mut Context) {}
}
