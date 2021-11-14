use riposte_common::{registry::Resource, utils::delimit_string};

pub fn resource_tooltip(resource: &Resource) -> String {
    let mut components = Vec::new();

    if resource.health_bonus > 0 {
        components.push(format!("+{} @icon{{health}}", resource.health_bonus));
    }
    if resource.happy_bonus > 0 {
        components.push(format!("+{} @icon{{happy}}", resource.happy_bonus));
    }

    delimit_string(&components, ", ")
}
