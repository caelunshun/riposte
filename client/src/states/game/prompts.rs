use std::collections::VecDeque;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::Game,
};

pub mod city_build;
pub mod research;

pub enum Action {
    Close,
}

pub trait Prompt: 'static {
    fn open(&mut self, cx: &mut Context, game: &Game, client: &mut Client<GameState>);

    fn update(
        &mut self,
        cx: &mut Context,
        game: &Game,
        client: &mut Client<GameState>,
    ) -> Option<Action>;
}

/// A queue of prompts to display to the user - these ask
/// for what to build in a city, what to research.
#[derive(Default)]
pub struct Prompts {
    queue: VecDeque<Box<dyn Prompt>>,
    current_prompt: Option<Box<dyn Prompt>>,
}

impl Prompts {
    pub fn push(&mut self, prompt: impl Prompt) {
        self.queue.push_back(Box::new(prompt));
    }

    pub fn is_empty(&self) -> bool {
        self.current_prompt.is_none() && self.queue.is_empty()
    }

    pub fn update(&mut self, cx: &mut Context, game: &Game, client: &mut Client<GameState>) {
        if let Some(prompt) = &mut self.current_prompt {
            if let Some(Action::Close) = prompt.update(cx, game, client) {
                self.current_prompt = None;
            }
        }

        if self.current_prompt.is_none() {
            self.current_prompt = self.queue.pop_front();

            if let Some(new_prompt) = &mut self.current_prompt {
                new_prompt.open(cx, game, client);
            }
        }
    }
}
