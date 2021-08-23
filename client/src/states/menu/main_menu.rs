use crate::{
    context::Context,
    generated::{MainMenu, MenuBackground, MenuEntry, UserBar},
    state::StateAttachment,
    ui::{FillScreen, Z_BACKGROUND, Z_FOREGROUND},
};
use ahash::AHashMap;

pub enum Action {
    PushOptions,
}

#[derive(Copy, Clone, Debug)]
enum Page {
    Main,
    Singleplayer,
}

#[derive(Debug, Copy, Clone)]

enum Message {
    SingleplayerClicked,
    OptionsClicked,
    BackClicked,
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
        handle.user_text.get_mut().set_text(
            "Account: %username - %uuid",
            vars! {
                username => cx.options().account().username(),
                uuid => cx.options().account().uuid().to_hyphenated(),
            },
        );

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
                    .add_entry(cx, "MULTIPLAYER", None)
                    .add_entry(cx, "OPTIONS", Some(Message::OptionsClicked));
            }
            Page::Singleplayer => {
                self.add_entry(cx, "NEW GAME", None)
                    .add_entry(cx, "LOAD GAME", None)
                    .add_entry(cx, "BACK", Some(Message::BackClicked));
            }
        }
    }

    fn add_entry(&self, cx: &Context, name: &str, message: Option<Message>) -> &Self {
        let (handle, entry) = cx.ui_mut().create_spec_instance::<MenuEntry>();
        handle.the_text.get_mut().set_text(name, AHashMap::new());

        if let Some(msg) = message {
            handle.clickable.get_mut().on_click(move || msg.clone());
        }

        self.handle.entries.get_mut().add_child(entry);
        self
    }
}
