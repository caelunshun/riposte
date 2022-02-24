use duit::{Align, Vec2};

use crate::{
    context::Context,
    game::{city::City, Game},
    generated::CityInfoBarWindow,
    state::StateAttachment,
    tooltips::{
        happiness::happiness_tooltip, health::health_tooltip, sickness::sickness_tooltip,
        unhappiness::unhappiness_tooltip,
    },
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([600., 200.]);

pub struct InfoBarScreen {
    window: CityInfoBarWindow,
}

impl InfoBarScreen {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<CityInfoBarWindow, _>(
            AlignFixed::new(SIZE, Align::Center, Align::Start),
            Z_FOREGROUND + 1,
        );

        Self { window }
    }

    pub fn update_info(&mut self, _cx: &Context, _game: &Game, city: &City) {
        self.window
            .city_name
            .get_mut()
            .set_text(text!("{}: {}", city.name(), city.population()));

        let growth_progress = city.stored_food() as f32 / city.food_needed_for_growth() as f32;
        let growth_projected_progress = (city.stored_food() as i32
            + city.economy().food_yield as i32
            - city.food_consumed_per_turn() as i32) as f32
            / city.food_needed_for_growth() as f32;
        self.window
            .growth_progress_bar
            .get_mut()
            .set_progress(growth_progress)
            .set_projected_progress(growth_projected_progress);

        let (prod_progress, prod_projected_progress) = match city.build_task() {
            Some(task) => {
                let progress = city.build_task_progress(task);
                (
                    progress as f32 / task.cost() as f32,
                    (progress + city.economy().hammer_yield) as f32 / task.cost() as f32,
                )
            }
            None => (0., 0.),
        };
        self.window
            .production_progress_bar
            .get_mut()
            .set_progress(prod_progress)
            .set_projected_progress(prod_projected_progress);

        self.window.food_text.get_mut().set_text(text!(
            "{} @icon[bread] - {} @icon[eaten_bread]",
            city.economy().food_yield,
            city.food_consumed_per_turn()
        ));
        self.window
            .hammers_text
            .get_mut()
            .set_text(text!("{} @icon[hammer]", city.economy().hammer_yield));

        let growth_text = if city.is_growing() {
            text!("Growing ({} turns)", city.turns_needed_for_growth())
        } else if city.is_stagnant() {
            text!("Stagnant")
        } else {
            text!("@color[180, 20, 30][STARVATION!]")
        };
        self.window.growth_text.get_mut().set_text(growth_text);

        let production_text = match city.build_task() {
            Some(task) => format!(
                "{} ({} turns)",
                task.name(),
                city.estimate_remaining_build_time()
            ),
            None => String::new(),
        };
        self.window
            .production_text
            .get_mut()
            .set_text(text!("{}", production_text));

        self.window
            .happy_text
            .get_mut()
            .set_text(text!("{}@icon[happy] ", city.num_happiness()));
        self.window
            .happy_sign_text
            .get_mut()
            .set_text(text!("{}", sign(city.num_happiness(), city.num_anger())));
        self.window
            .unhappy_text
            .get_mut()
            .set_text(text!(" {}@icon[unhappy]", city.num_anger()));

        self.window
            .health_text
            .get_mut()
            .set_text(text!("{}@icon[health] ", city.num_health()));
        self.window
            .health_sign_text
            .get_mut()
            .set_text(text!("{}", sign(city.num_health(), city.num_sickness())));
        self.window
            .sick_text
            .get_mut()
            .set_text(text!(" {}@icon[sick]", city.num_sickness()));

        self.window
            .happy_tooltip_text
            .get_mut()
            .set_text(text!("{}", happiness_tooltip(city)));
        self.window
            .unhappy_tooltip_text
            .get_mut()
            .set_text(text!("{}", unhappiness_tooltip(city)));
        self.window
            .health_tooltip_text
            .get_mut()
            .set_text(text!("{}", health_tooltip(city)));
        self.window
            .sick_tooltip_text
            .get_mut()
            .set_text(text!("{}", sickness_tooltip(city)));
    }
}

fn sign(a: u32, b: u32) -> &'static str {
    if a > b {
        ">"
    } else if a < b {
        "<"
    } else {
        "="
    }
}
