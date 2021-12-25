use duit::Vec2;
use glam::vec2;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::Game,
    generated::TechPopup,
    state::StateAttachment,
    tooltips::tech::tech_tooltip,
    ui::{Center, Z_POPUP},
};

use riposte_common::{assets::Handle, registry::Tech};

use super::{Action, Prompt};

pub const SIZE: Vec2 = glam::const_vec2!([600., 600.]);

struct Close;

pub struct TechPrompt {
    attachment: StateAttachment,

    window: Option<TechPopup>,

    tech: Handle<Tech>,
}

impl TechPrompt {
    pub fn new(cx: &Context, tech: Handle<Tech>) -> Self {
        Self {
            attachment: cx.state_manager().create_state(),
            window: None,
            tech,
        }
    }

    fn init(&mut self, cx: &Context, game: &Game) {
        let (window, _) = self
            .attachment
            .create_window::<TechPopup, _>(Center::with_size(vec2(600., 600.)), Z_POPUP);

        window
            .tech_name
            .get_mut()
            .set_text(text!("@bold[{}]", self.tech.name));
        window
            .tooltip_text
            .get_mut()
            .set_text(tech_tooltip(game.registry(), game, &self.tech));

        if let Some(quote) = &self.tech.quote {
            window.quote_text.get_mut().set_text(text!(
                "@size[16][\"{}\"\n\t —{}]",
                quote.text,
                quote.attribution
            ));
        }

        window.close_button.get_mut().on_click(move || Close);

        self.window = Some(window);
    }
}

impl Prompt for TechPrompt {
    fn open(&mut self, cx: &mut Context, game: &Game, _client: &mut Client<GameState>) {
        self.init(cx, game);
    }

    fn update(
        &mut self,
        cx: &mut Context,
        game: &Game,
        client: &mut Client<GameState>,
    ) -> Option<Action> {
        if cx.ui_mut().pop_message::<Close>().is_some() {
            Some(Action::Close)
        } else {
            None
        }
    }
}
