use duit::widgets::*;
use duit::*;
pub struct MenuEntry {
    pub clickable: WidgetHandle<Clickable>,
    pub the_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for MenuEntry {
    fn name() -> &'static str {
        "MenuEntry"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut clickable = None;
        let mut the_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "clickable" => clickable = Some(widget),
                "the_text" => the_text = Some(widget),
                _ => {}
            }
        }
        Self {
            clickable: WidgetHandle::new(clickable.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "clickable"
                )
            })),
            the_text: WidgetHandle::new(the_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "the_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct MainMenu {
    pub entries: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for MainMenu {
    fn name() -> &'static str {
        "MainMenu"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut entries = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "entries" => entries = Some(widget),
                _ => {}
            }
        }
        Self {
            entries: WidgetHandle::new(entries.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "entries"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct MenuBackground {}
impl ::duit::InstanceHandle for MenuBackground {
    fn name() -> &'static str {
        "MenuBackground"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        for (name, widget) in widget_handles {
            match name.as_str() {
                _ => {}
            }
        }
        Self {}
    }
}
