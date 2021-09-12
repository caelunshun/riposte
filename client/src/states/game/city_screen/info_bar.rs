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
        self.window.city_name.get_mut().set_text(
            "%city: %pop",
            vars! {
                city => city.name(),
                pop => city.population(),
            },
        );

        let growth_progress = city.stored_food() as f32 / city.food_needed_for_growth() as f32;
        let growth_projected_progress = (city.stored_food() + city.city_yield().food as i32
            - city.consumed_food()) as f32
            / city.food_needed_for_growth() as f32;
        self.window
            .growth_progress_bar
            .get_mut()
            .set_progress(growth_progress)
            .set_projected_progress(growth_projected_progress);

        let (prod_progress, prod_projected_progress) = match city.build_task() {
            Some(task) => (
                task.progress as f32 / task.cost as f32,
                (task.progress + city.city_yield().hammers) as f32 / task.cost as f32,
            ),
            None => (0., 0.),
        };
        self.window
            .production_progress_bar
            .get_mut()
            .set_progress(prod_progress)
            .set_projected_progress(prod_projected_progress);

        self.window.food_text.get_mut().set_text(
            format!(
                "{} @icon{{bread}} - {} @icon{{eaten_bread}}",
                city.city_yield().food,
                city.consumed_food()
            ),
            vars! {},
        );
        self.window.hammers_text.get_mut().set_text(
            format!("{} @icon{{hammer}}", city.city_yield().hammers),
            vars! {},
        );

        let growth_text = if city.is_growing() {
            format!("Growing ({} turns)", city.turns_needed_for_growth())
        } else if city.is_stagnant() {
            "Stagnant".to_owned()
        } else {
            "@color{rgb(180, 20, 30)}{STARVATION!}".to_owned()
        };
        self.window
            .growth_text
            .get_mut()
            .set_text(growth_text, vars! {});

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
            .set_text(production_text, vars! {});

        self.window
            .happy_text
            .get_mut()
            .set_text(format!("{}@icon{{happy}} ", city.num_happiness()), vars! {});
        self.window
            .happy_sign_text
            .get_mut()
            .set_text(sign(city.num_happiness(), city.num_unhappiness()), vars! {});
        self.window.unhappy_text.get_mut().set_text(
            format!(" {}@icon{{unhappy}}", city.num_unhappiness()),
            vars! {},
        );

        self.window
            .health_text
            .get_mut()
            .set_text(format!("{}@icon{{health}} ", city.num_health()), vars! {});
        self.window
            .health_sign_text
            .get_mut()
            .set_text(sign(city.num_health(), city.num_sickness()), vars! {});
        self.window
            .sick_text
            .get_mut()
            .set_text(format!(" {}@icon{{sick}}", city.num_sickness()), vars! {});

        self.window
            .happy_tooltip_text
            .get_mut()
            .set_text(happiness_tooltip(city), vars! {});
        self.window
            .unhappy_tooltip_text
            .get_mut()
            .set_text(unhappiness_tooltip(city), vars! {});
        self.window
            .health_tooltip_text
            .get_mut()
            .set_text(health_tooltip(city), vars! {});
        self.window
            .sick_tooltip_text
            .get_mut()
            .set_text(sickness_tooltip(city), vars! {});
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
