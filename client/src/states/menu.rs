use crate::{
    audio::{SoundCategory, SoundHandle},
    context::Context,
    generated::MenuBackground,
    options::Account,
    state::StateAttachment,
    ui::{FillScreen, Z_BACKGROUND},
    volumes,
};

use self::{login::LoginState, main_menu::MainMenuState, options::OptionsState};

mod login;
mod main_menu;
pub mod options;

enum State {
    MainMenu(MainMenuState),
    Login(LoginState),
    Options(OptionsState),
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

        Self {
            state: State::Login(LoginState::new(cx)),
            attachment,
            music,
        }
    }

    pub fn update(&mut self, cx: &mut Context) {
        match &mut self.state {
            State::MainMenu(s) => match s.update(cx) {
                Some(action) => match action {
                    main_menu::Action::PushOptions => {
                        self.state = State::Options(OptionsState::new(cx))
                    }
                },
                None => {}
            },
            State::Login(s) => {
                let authenticated = s.update(cx);
                if let Some(authenticated) = authenticated {
                    cx.options_mut()
                        .set_account(Account::from_authentication(authenticated));
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
            }
            State::Options(s) => {
                if let Some(options::Action::Close) = s.update(cx) {
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
            }
        }
    }
}
