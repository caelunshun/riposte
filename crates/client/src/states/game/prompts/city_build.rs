use duit::{Align, Vec2};
use glam::vec2;
use riposte_common::PossibleCityBuildTasks;

use crate::{
    client::{Client, GameState, ServerResponseFuture},
    context::Context,
    game::{
        city::{BuildTask, BuildTaskKind, PreviousBuildTask},
        CityId, Game,
    },
    generated::{CityBuildPromptOption, CityBuildPromptWindow},
    state::StateAttachment,
    tooltips,
    ui::{AlignFixed, Z_POPUP},
    utils::article,
};

use super::{Action, Prompt};

pub const SIZE: Vec2 = glam::const_vec2!([300., 500.]);

struct SetTask(riposte_common::BuildTask);

/// Asks the user what to build in a city.
pub struct CityBuildPrompt {
    attachment: StateAttachment,

    city: CityId,

    possible_tasks: ServerResponseFuture<PossibleCityBuildTasks>,

    window: Option<CityBuildPromptWindow>,

    received_tasks: bool,
}

impl CityBuildPrompt {
    pub fn new(cx: &Context, game: &Game, client: &mut Client<GameState>, city: CityId) -> Self {
        Self {
            attachment: cx.state_manager().create_state(),
            city,
            possible_tasks: client.get_possible_city_build_tasks(game, city),
            received_tasks: false,
            window: None,
        }
    }

    fn title_text(&self, game: &Game) -> String {
        let city = game.city(self.city);

        match city.previous_build_task() {
            Some(PreviousBuildTask { succeeded, task }) => {
                let (verb, verb_participle, name) = match &task.kind {
                    BuildTaskKind::Unit(unit) => ("trained", "training", &unit.name),
                    BuildTaskKind::Building(building) => {
                        ("constructed", "constructing", &building.name)
                    }
                };

                if *succeeded {
                    format!("You have {} {} @color{{rgb(255, 191, 63)}}{{{}}} in {}. What would you like to work on next?", 
                        verb, article(name), name, city.name())
                } else {
                    format!("You can no longer continue {} {} @color{{rgb(255, 191, 63)}}{{{}}} in {}. What would you like to work on instead?", 
                        verb_participle, article(name), name, city.name())
                }
            }
            None => format!("What would you like to build in {}?", city.name()),
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
            .set_text(self.title_text(game), vars! {});

        game.view_mut().animate_to(cx, city.pos());

        self.window = Some(window);
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

        if !self.received_tasks {
            if let Some(tasks) = self.possible_tasks.get() {
                self.initialize_tasks(cx, game, tasks);
                self.received_tasks = true;
            }
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
    fn initialize_tasks(&mut self, cx: &mut Context, game: &Game, tasks: PossibleCityBuildTasks) {
        let mut ui = cx.ui_mut();
        let city = game.city(self.city);
        for proto_task in tasks.tasks {
            match BuildTask::from_data(&proto_task, game) {
                Ok(task) => {
                    let (handle, widget) = ui.create_spec_instance::<CityBuildPromptOption>();

                    handle.option_text.get_mut().set_text(
                        "%name (%turns)",
                        vars! {
                            name => task.name(),
                            turns => city.estimate_build_time_for_task(&task),
                        },
                    );
                    handle
                        .clickable
                        .get_mut()
                        .on_click(move || SetTask(proto_task.clone()));

                    let tooltip_text = tooltips::build_task_tooltip(cx.registry(), &task.kind);
                    handle.tooltip_text.get_mut().set_text(
                        tooltip_text,
                        vars! {
                            percent => "%"
                        },
                    );

                    self.window
                        .as_mut()
                        .unwrap()
                        .options_column
                        .get_mut()
                        .add_child(widget);
                }
                Err(e) => log::error!("Received bad build task from server: {}", e),
            }
        }
    }
}
