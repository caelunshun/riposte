use crate::{
    audio::SoundHandle,
    context::Context,
    generated::MenuBackground,
    options::Account,
    state::StateAttachment,
    ui::{FillScreen, Z_BACKGROUND},
    volumes,
};

use self::{login::LoginState, main_menu::MainMenuState};

mod login;
mod main_menu;

enum State {
    MainMenu(MainMenuState),
    Login(LoginState),
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

        let music = cx.audio().play_looping("music/menu", volumes::MENU_MUSIC);

        Self {
            state: State::Login(LoginState::new(cx)),
            attachment,
            music,
        }
    }

    pub fn update(&mut self, cx: &mut Context) {
        match &mut self.state {
            State::MainMenu(s) => s.update(cx),
            State::Login(s) => {
                let authenticated = s.update(cx);
                if let Some(authenticated) = authenticated {
                    cx.options_mut()
                        .set_account(Account::from_authentication(authenticated));
                    self.state = State::MainMenu(MainMenuState::new(cx));
                }
            }
        }
    }
}
