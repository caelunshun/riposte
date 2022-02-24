use duit::widgets::*;
use duit::{WidgetHandle, WidgetPodHandle};
pub struct RiposteLauncher {
    pub progress_bar: WidgetHandle<ProgressBar>,
    pub progress_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for RiposteLauncher {
    fn name() -> &'static str {
        "RiposteLauncher"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut progress_bar = None;
        let mut progress_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "progress_bar" => progress_bar = Some(widget),
                "progress_text" => progress_text = Some(widget),
                _ => {}
            }
        }
        Self {
            progress_bar: WidgetHandle::new(progress_bar.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "progress_bar"
                )
            })),
            progress_text: WidgetHandle::new(progress_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "progress_text"
                )
            })),
        }
    }
}
