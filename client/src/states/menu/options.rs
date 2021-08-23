use duit::{widgets::Slider, WidgetHandle};

use crate::{
    context::Context,
    generated,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

struct BackClicked;

pub enum Action {
    Close,
}

/// Displays a window to configure game settings.
pub struct OptionsState {
    attachment: StateAttachment,

    handle: generated::OptionsWindow,
}

fn write_option_to_slider(option: f32, slider: &WidgetHandle<Slider>) {
    slider.get_mut().set_value(option);
}

fn get_option_from_slider(option: &mut f32, slider: &WidgetHandle<Slider>) -> bool {
    let new_value = slider.get().value();
    let changed = *option != new_value;
    *option = new_value;
    changed
}

impl OptionsState {
    pub fn new(cx: &Context) -> Self {
        let attachment = cx.state_manager().create_state();

        let (handle, _) =
            attachment.create_window::<generated::OptionsWindow, _>(FillScreen, Z_FOREGROUND);

        let options = cx.options();
        let sound_options = options.sound();
        write_option_to_slider(sound_options.music_volume, &handle.music_volume_slider);
        write_option_to_slider(sound_options.effects_volume, &handle.effects_volume_slider);

        handle.back_button.get_mut().on_click(|| BackClicked);

        Self { attachment, handle }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Action> {
        if cx.ui_mut().pop_message::<BackClicked>().is_some() {
            cx.save_options_to_disk();
            return Some(Action::Close);
        }

        // Update options
        let mut changed = false;
        let mut options = cx.options_mut();
        let sound_options = options.sound_mut();
        changed |= get_option_from_slider(
            &mut sound_options.music_volume,
            &self.handle.music_volume_slider,
        );
        changed |= get_option_from_slider(
            &mut sound_options.effects_volume,
            &self.handle.effects_volume_slider,
        );

        if changed {
            drop(options);
            cx.audio_mut().on_sound_options_changed();
        }

        None
    }
}
