use ahash::AHashMap;
use duit::Vec2;
use dume::{
    font::{Query, Weight},
    Align, Baseline, Canvas, Text, TextBlob, TextOptions, TextSection, TextStyle, TextureId,
};
use glam::{vec2, UVec2};
use palette::{Shade, Srgba};

use crate::{
    context::Context,
    game::{
        city::{BuildTask, City},
        Game, Tile,
    },
};

use super::TileRenderLayer;

pub struct CityRenderer {
    house: TextureId,

    /// Used to display the first letter of a building's name, as
    /// well as the city population.
    letter_blobs: AHashMap<String, TextBlob>,

    /// Used for the production progress bar and city name.
    text_blobs: AHashMap<String, TextBlob>,

    unit_heads: AHashMap<String, TextureId>,
}

const BUBBLE_SIZE: Vec2 = glam::const_vec2!([100., 20.]);

impl CityRenderer {
    pub fn new(cx: &Context) -> Self {
        let house = cx
            .canvas()
            .context()
            .texture_for_name("icon/house")
            .unwrap();

        let mut unit_heads = AHashMap::new();
        for unit in cx.registry().unit_kinds() {
            let unit_head = cx
                .canvas()
                .context()
                .texture_for_name(&format!("icon/unit_head/{}", unit.id))
                .unwrap_or_else(|_| panic!("missing unit head icon for unit '{}", unit.id));
            unit_heads.insert(unit.id.clone(), unit_head);
        }

        Self {
            house,

            letter_blobs: AHashMap::new(),
            text_blobs: AHashMap::new(),

            unit_heads,
        }
    }

    fn letter_blob(&mut self, canvas: &mut Canvas, c: String) -> &TextBlob {
        self.letter_blobs.entry(c.clone()).or_insert_with(move || {
            let text = Text::from_sections(vec![TextSection::Text {
                text: c.into(),
                style: TextStyle {
                    color: Some(Srgba::new(0, 0, 0, u8::MAX)),
                    size: Some(18.),
                    font: Query {
                        weight: Weight::Light,
                        ..Default::default()
                    },
                },
            }]);
            let mut blob = canvas.context().create_text_blob(
                &text,
                TextOptions {
                    wrap_lines: false,
                    baseline: Baseline::Bottom,
                    align_h: Align::Center,
                    align_v: Align::Start,
                },
            );
            canvas
                .context()
                .resize_text_blob(&mut blob, vec2(20., f32::INFINITY));
            blob
        })
    }

    fn text_blob(&mut self, canvas: &mut Canvas, text: String) -> &TextBlob {
        self.letter_blobs
            .entry(text.clone())
            .or_insert_with(move || {
                let text = Text::from_sections(vec![TextSection::Text {
                    text: text.into(),
                    style: TextStyle {
                        color: Some(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX)),
                        size: Some(8.),
                        font: Query {
                            weight: Weight::Light,
                            ..Default::default()
                        },
                    },
                }]);
                let mut blob = canvas.context().create_text_blob(
                    text,
                    TextOptions {
                        wrap_lines: false,
                        baseline: Baseline::Top,
                        align_h: Align::Center,
                        align_v: Align::Start,
                    },
                );
                canvas
                    .context()
                    .resize_text_blob(&mut blob, vec2(100., f32::INFINITY));
                blob
            })
    }

    fn render_houses(&mut self, canvas: &mut Canvas) {
        let house_positions = [vec2(20., 25.), vec2(50., 25.), vec2(25., 30.)];
        let house_scales = [25., 25., 55.];

        for (&pos, scale) in house_positions.iter().zip(house_scales) {
            canvas.draw_sprite(self.house, pos, scale / 1.424);
        }
    }

    fn render_progress_bar(
        &mut self,
        canvas: &mut Canvas,
        pos: Vec2,
        size: Vec2,
        progress: f32,
        projected_progress: f32,
        progress_color: Srgba<u8>,
        projected_progress_color: Srgba<u8>,
        text: String,
    ) {
        let progress = progress.clamp(0., 1.);
        let projected_progress = projected_progress.clamp(0., 1.);

        canvas
            .begin_path()
            .rect(pos, vec2(size.x * progress, size.y))
            .solid_color(progress_color)
            .fill();

        canvas
            .begin_path()
            .rect(
                pos + vec2(size.x * progress, 0.),
                vec2(size.x * (projected_progress - progress), size.y),
            )
            .solid_color(projected_progress_color)
            .fill();

        let text = self.text_blob(canvas, text);
        canvas.draw_text(text, pos, 1.);
    }

    fn render_production_progress_bar(&mut self, canvas: &mut Canvas, city: &City) {
        if let Some(task) = city.build_task() {
            let progress = city.build_task_progress(task) as f32 / task.cost() as f32;
            let projected_progress = (city.build_task_progress(task) + city.economy().hammer_yield)
                as f32
                / task.cost() as f32;
            self.render_progress_bar(
                canvas,
                vec2(0., 80.0),
                vec2(BUBBLE_SIZE.x, BUBBLE_SIZE.y / 2.),
                progress,
                projected_progress,
                Srgba::new(72, 159, 223, 160),
                Srgba::new(141, 200, 232, 160),
                format!("{} ({})", task.name(), city.estimate_remaining_build_time()),
            );
        }
    }

    fn render_population_progress_bar(&mut self, game: &Game, canvas: &mut Canvas, city: &City) {
        let mut progress = city.stored_food() as f32 / city.food_needed_for_growth() as f32;
        let mut projected_progress = (city.stored_food() as i32 + city.economy().food_yield as i32
            - city.food_consumed_per_turn() as i32) as f32
            / city.food_needed_for_growth() as f32;
        if city.owner() != game.the_player().id() {
            progress = 0.;
            projected_progress = 0.;
        }
        self.render_progress_bar(
            canvas,
            vec2(0., 70.),
            vec2(BUBBLE_SIZE.x, BUBBLE_SIZE.y / 2.),
            progress,
            projected_progress,
            Srgba::new(237, 155, 51, u8::MAX),
            Srgba::new(185, 112, 0, u8::MAX),
            city.name().to_owned(),
        );
    }

    fn render_left_circle(&mut self, game: &Game, canvas: &mut Canvas, city: &City) {
        // Circle or star
        let radius = 10.;
        let center = vec2(radius - 5., radius + 70.);

        canvas.begin_path();
        if city.is_capital() {
            super::five_point_star(canvas, center, 18., 8.);
        } else {
            canvas.circle(center, radius);
        }

        let color = if city.owner() != game.the_player().id() {
            // Brighten the civ's color so it contrasts with black text
            let col = game.player(city.owner()).civ().color.clone();
            Srgba::new(col[0], col[1], col[2], u8::MAX)
                .into_format::<f32, f32>()
                .into_linear()
                .lighten(0.5)
                .into_encoding()
                .into_format()
        } else if city.is_growing() {
            Srgba::new(182, 207, 174, u8::MAX)
        } else if city.is_starving() {
            Srgba::new(231, 60, 62, u8::MAX)
        } else {
            Srgba::new(200, 200, 200, u8::MAX)
        };

        canvas.solid_color(color).fill();
        canvas
            .solid_color(Srgba::new(0, 0, 0, u8::MAX))
            .stroke_width(1.5)
            .stroke();

        let population = lexical::to_string(city.population().get());
        let population_text = self.letter_blob(canvas, population);
        canvas.draw_text(population_text, vec2(-5., 80.), 1.);
    }

    fn render_right_circle(&mut self, canvas: &mut Canvas, city: &City) {
        let radius = 10.;
        canvas
            .begin_path()
            .circle(vec2(-radius + 5. + BUBBLE_SIZE.x, radius + 70.), radius)
            .solid_color(Srgba::new(244, 195, 204, u8::MAX))
            .fill();
        canvas
            .solid_color(Srgba::new(0, 0, 0, u8::MAX))
            .stroke_width(1.5)
            .stroke();

        if let Some(task) = city.build_task() {
            let pos = vec2(-radius * 2. + 5. + BUBBLE_SIZE.x, 70.);
            match task {
                BuildTask::Unit(unit) => {
                    canvas.draw_sprite(self.unit_heads[&unit.id], pos, radius * 2.);
                }
                BuildTask::Building(building) => {
                    // First character of the building name
                    let text = self.letter_blob(
                        canvas,
                        building.name.chars().next().unwrap_or('0').to_string(),
                    );
                    canvas.draw_text(text, pos + vec2(0., radius), 1.);
                }
            }
        }
    }

    fn render_bubble(&mut self, game: &Game, canvas: &mut Canvas, city: &City) {
        // Bubble background
        canvas
            .begin_path()
            .rounded_rect(vec2(0., 70.), BUBBLE_SIZE, 5.)
            .linear_gradient(
                vec2(0., 70.),
                vec2(0., 90.),
                Srgba::new(61, 61, 62, 180),
                Srgba::new(40, 40, 41, 180),
            )
            .fill();

        if city.owner() == game.the_player().id() {
            self.render_production_progress_bar(canvas, city);
        }
        self.render_population_progress_bar(game, canvas, city);

        self.render_left_circle(game, canvas, city);

        if city.owner() == game.the_player().id() {
            self.render_right_circle(canvas, city);
        }
    }
}

impl TileRenderLayer for CityRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, _tile: &Tile) {
        let city = match game.city_at_pos(tile_pos) {
            Some(c) => c,
            None => return,
        };

        let mut canvas = cx.canvas_mut();

        self.render_houses(&mut canvas);
        self.render_bubble(game, &mut canvas, &city);
    }
}
