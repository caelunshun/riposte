use crate::{
    audio::SoundHandle,
    context::Context,
    generated::{MainMenu, MenuBackground, MenuEntry},
    state::StateAttachment,
    ui::{FillScreen, Z_BACKGROUND, Z_FOREGROUND},
    volumes,
};
use ahash::AHashMap;

#[derive(Copy, Clone, Debug)]
enum Page {
    Main,
    Singleplayer,
}

#[derive(Debug, Copy, Clone)]
enum Message {
    SingleplayerClicked,
    BackClicked,
}

pub struct MainMenuState {
    attachment: StateAttachment,
    music: SoundHandle,
    handle: MainMenu,
    current_page: Page,
}

impl MainMenuState {
    pub fn new(cx: &Context) -> Self {
        let attachment = cx.state_manager().create_state();

        attachment.create_window::<MenuBackground, _>(FillScreen, Z_BACKGROUND);
        let (handle, _) = attachment.create_window::<MainMenu, _>(FillScreen, Z_FOREGROUND);

        let music = cx.audio().play_looping("music/menu", volumes::MENU_MUSIC);

        let this = Self {
            handle,
            attachment,
            music,
            current_page: Page::Main,
        };

        this.update_entries(cx);

        this
    }

    pub fn update(&mut self, cx: &Context) {
        let mut updated = false;
        cx.ui_mut().handle_messages(|msg: &Message| {
            updated = true;
            match msg {
                Message::SingleplayerClicked => self.current_page = Page::Singleplayer,
                Message::BackClicked => self.current_page = Page::Main,
            }
        });

        if updated {
            self.update_entries(cx);
        }
    }

    fn update_entries(&self, cx: &Context) {
        self.handle.entries.get_mut().clear_children();

        match self.current_page {
            Page::Main => {
                self.add_entry(cx, "SINGLEPLAYER", Some(Message::SingleplayerClicked))
                    .add_entry(cx, "MULTIPLAYER", None)
                    .add_entry(cx, "OPTIONS", None);
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
        handle
            .the_text
            .get_mut()
            .set_text(name.into(), AHashMap::new());

        if let Some(msg) = message {
            handle.clickable.get_mut().on_click(move || msg.clone());
        }

        self.handle.entries.get_mut().add_child(entry);
        self
    }
}
