use duit::{Align, Vec2};
use dume::Text;
use glam::vec2;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::{city::BuildTask, Game},
    generated::{CityBuildPromptOption, CityBuildPromptWindow},
    state::StateAttachment,
    tooltips,
    ui::{AlignFixed, Z_POPUP},
};

use riposte_common::{city::PreviousBuildTask, utils::article, CityId};

use super::{Action, Prompt};

pub const SIZE: Vec2 = glam::const_vec2!([300., 500.]);

struct SetTask(BuildTask);

/// Asks the user what to build in a city.
pub struct CityBuildPrompt {
    attachment: StateAttachment,

    city: CityId,

    window: Option<CityBuildPromptWindow>,
}

impl CityBuildPrompt {
    pub fn new(_game: &Game, cx: &Context, city: CityId) -> Self {
        Self {
            attachment: cx.state_manager().create_state(),
            city,
            window: None,
        }
    }

    fn title_text(&self, game: &Game) -> Text {
        let city = game.city(self.city);

        match city.previous_build_task() {
            Some(PreviousBuildTask { success, task }) => {
                let (verb, verb_participle, name) = match &task {
                    BuildTask::Unit(unit) => ("trained", "training", &unit.name),
                    BuildTask::Building(building) => {
                        ("constructed", "constructing", &building.name)
                    }
                };

                if *success {
                    text!("You have {} {} @color[255, 191, 63][{}] in {}. What would you like to work on next?", 
                        verb, article(name), name, city.name())
                } else {
                    text!("You can no longer continue {} {} @color[255, 191, 63][{}] in {}. What would you like to work on instead?", 
                        verb_participle, article(name), name, city.name())
                }
            }
            None => text!("What would you like to build in {}?", city.name()),
        }
    }
}

impl Prompt for CityBuildPrompt {
    fn open(&mut self, cx: &mut Context, game: &Game, _client: &mut Client<GameState>) {
        if game.city(self.city).build_task().is_some() {
            return;
        }

        let (window, _) = self.attachment.create_window::<CityBuildPromptWindow, _>(
            AlignFixed::new(SIZE, Align::End, Align::Start).with_offset(vec2(-10., 120.)),
            Z_POPUP,
        );

        let city = game.city(self.city);

        window
            .question_text
            .get_mut()
            .set_text(self.title_text(game));

        game.view_mut().animate_to(cx, city.pos());

        self.window = Some(window);

        self.initialize_tasks(cx, game);
    }

    fn update(
        &mut self,
        cx: &mut Context,
        game: &Game,
        client: &mut Client<GameState>,
    ) -> Option<super::Action> {
        if game.city(self.city).build_task().is_some() {
            // We're not needed anymore.
            return Some(Action::Close);
        }

        if let Some(msg) = cx.ui_mut().pop_message::<SetTask>() {
            client.set_city_build_task(game, self.city, msg.0);
            Some(Action::Close)
        } else {
            None
        }
    }
}

impl CityBuildPrompt {
    fn initialize_tasks(&mut self, cx: &Context, game: &Game) {
        let tasks = game.city(self.city).possible_build_tasks(game.base());
        let mut ui = cx.ui_mut();
        let city = game.city(self.city);
        for task in &tasks {
            let (handle, widget) = ui.create_spec_instance::<CityBuildPromptOption>();

            handle.option_text.get_mut().set_text(text!(
                "{} ({})",
                task.name(),
                city.estimate_build_time_for_task(&task)
            ));

            {
                let task = task.clone();
                handle
                    .clickable
                    .get_mut()
                    .on_click(move || SetTask(task.clone()));
            }

            let tooltip_text = tooltips::build_task_tooltip(cx.registry(), &task);
            handle
                .tooltip_text
                .get_mut()
                .set_text(tooltip_text);

            self.window
                .as_mut()
                .unwrap()
                .options_column
                .get_mut()
                .add_child(widget);
        }
    }
}
