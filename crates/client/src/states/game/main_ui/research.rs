use duit::{Align, Vec2};
use glam::vec2;

use crate::{
    context::Context,
    game::{event::GameEvent, Game},
    generated::ResearchBarWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 30.]);

pub struct ResearchBar {
    window: ResearchBarWindow,
}

impl ResearchBar {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<ResearchBarWindow, _>(
            AlignFixed::new(SIZE, Align::Center, Align::Start).with_offset(vec2(0., 1.)),
            Z_FOREGROUND,
        );

        Self { window }
    }

    pub fn handle_game_event(&mut self, _cx: &Context, game: &Game, event: &GameEvent) {
        if let GameEvent::PlayerUpdated { player } = event {
            if *player == game.the_player().id() {
                self.update_info(game);
            }
        }
    }

    pub fn update_info(&mut self, game: &Game) {
        let the_player = game.the_player();
        let research = the_player.researching_tech();

        let (text, progress, projected_progress) = match research {
            Some(research) => {
                let progress = the_player.tech_progress(research);
                (
                    format!(
                        "Research: {} ({})",
                        research.name,
                        the_player.estimate_current_research_turns()
                    ),
                    progress as f32 / research.cost as f32,
                    (progress + the_player.beaker_revenue() as u32) as f32 / research.cost as f32,
                )
            }
            None => ("Research: None".to_owned(), 0., 0.),
        };

        self.window
            .research_progress
            .get_mut()
            .set_progress(progress)
            .set_projected_progress(projected_progress);
        self.window
            .research_text
            .get_mut()
            .set_text(text!("{}", text));
    }
}
