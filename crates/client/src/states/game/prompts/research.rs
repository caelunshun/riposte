use duit::{Align, Vec2};
use glam::vec2;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::Game,
    generated::{ResearchPromptOption, ResearchPromptWindow},
    state::StateAttachment,
    tooltips::tech::tech_tooltip,
    ui::{AlignFixed, Z_POPUP},
};

use riposte_common::{assets::Handle, registry::Tech};

use super::{Action, Prompt};

pub const SIZE: Vec2 = glam::const_vec2!([400., 400.]);

struct SetResearch(Handle<Tech>);

pub struct ResearchPrompt {
    attachment: StateAttachment,

    window: Option<ResearchPromptWindow>,
}

impl ResearchPrompt {
    pub fn new(cx: &Context, _client: &mut Client<GameState>) -> Self {
        Self {
            attachment: cx.state_manager().create_state(),
            window: None,
        }
    }

    fn init_with_techs(&mut self, cx: &Context, game: &Game, techs: Vec<Handle<Tech>>) {
        let window = self.window.as_mut().unwrap();

        for tech in techs {
            let (option, widget) = cx.ui_mut().create_spec_instance::<ResearchPromptOption>();

            let turns = game.the_player().estimate_research_turns(&tech, 0);
            option
                .option_text
                .get_mut()
                .set_text(text!("{} ({})", tech.name, turns));

            let tooltip = tech_tooltip(cx.registry(), game, &tech);
            option.tooltip_text.get_mut().set_text(tooltip);

            option
                .clickable
                .get_mut()
                .on_click(move || SetResearch(tech.clone()));

            window.options_column.get_mut().add_child(widget);
        }
    }
}

impl Prompt for ResearchPrompt {
    fn open(&mut self, cx: &mut Context, game: &Game, _client: &mut Client<GameState>) {
        if game.the_player().researching_tech().is_some() {
            return;
        }

        let (window, _) = self.attachment.create_window::<ResearchPromptWindow, _>(
            AlignFixed::new(SIZE, Align::Center, Align::Center).with_offset(vec2(0., -100.)),
            Z_POPUP,
        );
        self.window = Some(window);

        self.init_with_techs(cx, game, game.the_player().researchable_techs(game.base()));
    }

    fn update(
        &mut self,
        cx: &mut Context,
        game: &Game,
        client: &mut Client<GameState>,
    ) -> Option<Action> {
        if game.the_player().researching_tech().is_some() {
            return Some(Action::Close);
        }

        if let Some(msg) = cx.ui_mut().pop_message::<SetResearch>() {
            client.set_research(&msg.0);
            return Some(Action::Close);
        }

        None
    }
}
