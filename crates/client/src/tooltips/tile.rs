use std::mem;
use std::{ num::NonZeroUsize};

use dume::Text;
use glam::UVec2;
use lexical::{format::STANDARD, WriteFloatOptions};
use riposte_common::assets::Handle;
use riposte_common::registry::{Resource, UnitKind};
use riposte_common::unit::MovementPoints;
use riposte_common::{Improvement, PlayerId, UnitId, Visibility, Yield};

use crate::game::unit::{Capability, Unit};
use crate::game::{Game, Tile};
use crate::utils::{convert_color, delimit_text, merge_text_lines};

pub fn tile_tooltip(game: &Game, tile: &Tile, pos: UVec2) -> Text {
    let mut lines = Vec::new();

    lines.extend(culture_lines(game, tile));
    lines.extend(units_lines(game, tile, pos));
    lines.push(header(tile));
    lines.extend(defense_bonus_line(tile));
    lines.extend(improvement_lines(tile));
    lines.push(yield_description_line(&tile.tile_yield()));
    if let Some(resource) = tile.resource() {
        lines.extend(resource_line(game, tile, &resource));
    }

    merge_text_lines(lines)
}

fn culture_lines(game: &Game, tile: &Tile) -> Vec<Text> {
    let mut lines = Vec::new();

    let total_culture = tile.culture().iter().map(|c| c.amount()).sum::<u32>();

    if total_culture == 0 {
        return Vec::new();
    }

    for culture_value in tile.culture().iter() {
        let player = game.player(culture_value.owner());
        let percent = (culture_value.amount() as f64 / total_culture as f64 * 100.).floor() as u32;

        if percent != 0 {
            lines.push(text!(
                "{}% @color[{}][{}]",
                percent,
                convert_color(&player.civ().color),
                player.civ().adjective
            ));
        }
    }

    lines
}

struct UnitLinesEntry {
    kind: Handle<UnitKind>,
    movement_left: MovementPoints,
    strength: f64,
    owner: PlayerId,
    units: Vec<UnitId>,
}

impl UnitLinesEntry {
    pub fn matches_unit(&self, unit: &Unit) -> bool {
        unit.movement_left() == self.movement_left
            && unit.strength() == self.strength
            && unit.owner() == self.owner
            && unit.kind().id == self.kind.id
    }

    pub fn new(unit: &Unit) -> Self {
        Self {
            kind: unit.kind().clone(),
            movement_left: unit.movement_left(),
            strength: unit.strength(),
            owner: unit.owner(),
            units: vec![unit.id()],
        }
    }
}

fn units_lines(game: &Game, _tile: &Tile, pos: UVec2) -> Vec<Text> {
    if game.the_player().visibility_at(pos) != Visibility::Visible && !game.cheat_mode {
        return Vec::new();
    }

    // Coalesce units with the same attributes into the same line
    let mut entries: Vec<UnitLinesEntry> = Vec::new();
    for unit in game.unit_stack(pos).unwrap().units() {
        let unit = game.unit(*unit);

        match entries.iter_mut().find(|e| e.matches_unit(&unit)) {
            Some(e) => e.units.push(unit.id()),
            None => entries.push(UnitLinesEntry::new(&unit)),
        }
    }

    // Expand entries with <= 3 units for clarity
    for i in (0..entries.len()).rev() {
        let entry = &mut entries[i];
        if entry.units.len() <= 3 && entry.units.len() > 1 {
            for unit in mem::take(&mut entry.units) {
                entries.push(UnitLinesEntry::new(&game.unit(unit)));
            }
            entries.remove(i);
        }
    }

    let mut lines = Vec::new();

    let float_options = WriteFloatOptions::builder()
        .max_significant_digits(Some(NonZeroUsize::new(2).unwrap()))
        .trim_floats(true)
        .build()
        .unwrap();

    for entry in entries {
        let mut line = text!("@color[255,205,0][{}]", entry.kind.name);

        if entry.units.len() > 1 {
            line.extend(text!(" ({})", entry.units.len()));
        }

        if entry.kind.strength > 0. {
            let strength = if entry.strength == entry.kind.strength {
                lexical::to_string_with_options::<_, STANDARD>(entry.strength, &float_options)
            } else {
                format!(
                    "{}/{}",
                    lexical::to_string_with_options::<_, STANDARD>(entry.strength, &float_options),
                    lexical::to_string_with_options::<_, STANDARD>(
                        entry.kind.strength,
                        &float_options
                    )
                )
            };
            line.extend(text!(", {} @icon[strength]", strength));
        }

        let movement = if entry.movement_left.as_f64().ceil() as u32 == entry.kind.movement {
            entry.kind.movement.to_string()
        } else {
            format!(
                "{}/{}",
                entry.movement_left.as_f64().ceil() as u32,
                entry.kind.movement
            )
        };
        line.extend(text!(", {} @icon[movement]", movement));

        if entry.owner != game.the_player().id() {
            let owner = game.player(entry.owner);

            line.extend(text!(
                ", @color[{}][{}]",
                convert_color(&owner.civ().color),
                owner.username()
            ));
        } else {
            let unit = game.unit(entry.units.first().copied().unwrap());
            if let Some(worker_cap) = unit.capabilities().find_map(|c| match c {
                Capability::Worker(w) => Some(w),
                _ => None,
            }) {
                if let Some(task) = worker_cap.current_task() {
                    line.extend(text!(
                        ", {} ({})",
                        task.kind.present_participle(),
                        task.turns_left()
                    ));
                }
            };
        }

        lines.push(line);
    }

    lines
}

fn header(tile: &Tile) -> Text {
    let mut header = text!("{:?}", tile.terrain());
    if tile.is_hilled() {
        header.extend(text!(", Hills"));
    }
    if tile.is_forested() {
        header.extend(text!(", Forest"));
    }
    header
}

fn defense_bonus_line(tile: &Tile) -> Option<Text> {
    let bonus = tile.defense_bonus();
    if bonus > 0 {
        Some(text!("Defense bonus: +{}%", bonus))
    } else {
        None
    }
}

fn yield_description_line(yiel: &Yield) -> Text {
    let mut parts = Vec::new();

    if yiel.food > 0 {
        parts.push(text!("{}@icon[bread]", yiel.food));
    }
    if yiel.hammers > 0 {
        parts.push(text!("{}@icon[hammer]", yiel.hammers));
    }
    if yiel.commerce > 0 {
        parts.push(text!("{}@icon[coin]", yiel.commerce));
    }

    delimit_text(parts, text!(", "))
}

fn resource_line(game: &Game, tile: &Tile, resource: &Resource) -> Option<Text> {
    if !game
        .the_player()
        .has_unlocked_tech(&game.registry().tech(&resource.revealed_by).unwrap())
    {
        return None;
    }

    let mut line = text!("{}, ", resource.name);

    line.extend(yield_description_line(&resource.improved_bonus));

    if !tile
        .improvements()
        .any(|i| resource.improvement == i.name())
    {
        line.extend(text!(
            " (@color[200,30,60][Requires {})]",
            resource.improvement
        ));
    }

    Some(line)
}

fn improvement_lines(tile: &Tile) -> Vec<Text> {
    let mut lines = Vec::new();

    for improvement in tile.improvements() {
        match improvement {
            Improvement::Cottage(cottage) => {
                lines.push(text!("{:?}", cottage.level()));
                if !tile.is_worked() {
                    lines
                        .last_mut()
                        .unwrap()
                        .extend(text!(" (City must work to grow)"));
                }
            }
            _ => lines.push(text!("{}", improvement.name())),
        }
    }

    lines
}
