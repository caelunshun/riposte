use riposte_common::{utils::merge_lines, Improvement};

use crate::game::Tile;

pub fn build_improvement_tooltip(tile: &Tile, improvement: &Improvement) -> String {
    let mut lines = Vec::new();
    match improvement {
        Improvement::Farm => lines.push("+1 @icon{bread}".to_owned()),
        Improvement::Mine => lines.push("+1 @icon{hammer}".to_owned()),
        Improvement::Road => lines.push("Unit movement costs reduced by 1/3".to_owned()),
        Improvement::Pasture => {}
        Improvement::Cottage(_) => lines.extend(vec![
            "+1 @icon{coin}".to_owned(),
            "Grows for increased bonuses".to_owned(),
        ]),
        Improvement::Plantation => {}
    }

    if let Some(resource) = tile.resource() {
        if resource.improvement == improvement.name() {
            lines.push(format!(
                "@color{{rgb(30, 200, 50)}}{{Gives access to {}}}",
                resource.name
            ));
        }
    }

    merge_lines(&lines)
}
