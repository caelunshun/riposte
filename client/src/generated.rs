use duit::widgets::*;
use duit::*;
pub struct ErrorPopup {
    pub error_text: WidgetHandle<Text>,
    pub close_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for ErrorPopup {
    fn name() -> &'static str {
        "ErrorPopup"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut error_text = None;
        let mut close_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "error_text" => error_text = Some(widget),
                "close_button" => close_button = Some(widget),
                _ => {}
            }
        }
        Self {
            error_text: WidgetHandle::new(error_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "error_text"
                )
            })),
            close_button: WidgetHandle::new(close_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "close_button"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct GameLobbyWindow {
    pub add_ai_slot_button: WidgetHandle<Button>,
    pub add_human_slot_button: WidgetHandle<Button>,
    pub slots_table: WidgetHandle<Table>,
    pub start_game_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for GameLobbyWindow {
    fn name() -> &'static str {
        "GameLobbyWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut add_ai_slot_button = None;
        let mut add_human_slot_button = None;
        let mut slots_table = None;
        let mut start_game_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "add_ai_slot_button" => add_ai_slot_button = Some(widget),
                "add_human_slot_button" => add_human_slot_button = Some(widget),
                "slots_table" => slots_table = Some(widget),
                "start_game_button" => start_game_button = Some(widget),
                _ => {}
            }
        }
        Self {
            add_ai_slot_button: WidgetHandle::new(add_ai_slot_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "add_ai_slot_button"
                )
            })),
            add_human_slot_button: WidgetHandle::new(add_human_slot_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "add_human_slot_button"
                )
            })),
            slots_table: WidgetHandle::new(slots_table.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "slots_table"
                )
            })),
            start_game_button: WidgetHandle::new(start_game_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "start_game_button"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct UnitInfoWindow {
    pub header_text: WidgetHandle<Text>,
    pub info_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for UnitInfoWindow {
    fn name() -> &'static str {
        "UnitInfoWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut header_text = None;
        let mut info_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "header_text" => header_text = Some(widget),
                "info_text" => info_text = Some(widget),
                _ => {}
            }
        }
        Self {
            header_text: WidgetHandle::new(header_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "header_text"
                )
            })),
            info_text: WidgetHandle::new(info_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "info_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct OptionsWindow {
    pub music_volume_slider: WidgetHandle<Slider>,
    pub effects_volume_slider: WidgetHandle<Slider>,
    pub back_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for OptionsWindow {
    fn name() -> &'static str {
        "OptionsWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut music_volume_slider = None;
        let mut effects_volume_slider = None;
        let mut back_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "music_volume_slider" => music_volume_slider = Some(widget),
                "effects_volume_slider" => effects_volume_slider = Some(widget),
                "back_button" => back_button = Some(widget),
                _ => {}
            }
        }
        Self {
            music_volume_slider: WidgetHandle::new(music_volume_slider.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "music_volume_slider"
                )
            })),
            effects_volume_slider: WidgetHandle::new(effects_volume_slider.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "effects_volume_slider"
                )
            })),
            back_button: WidgetHandle::new(back_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "back_button"
                )
            })),
        }
    }
}
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
pub struct UserBar {
    pub user_text: WidgetHandle<Text>,
    pub log_out_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for UserBar {
    fn name() -> &'static str {
        "UserBar"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut user_text = None;
        let mut log_out_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "user_text" => user_text = Some(widget),
                "log_out_button" => log_out_button = Some(widget),
                _ => {}
            }
        }
        Self {
            user_text: WidgetHandle::new(user_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "user_text"
                )
            })),
            log_out_button: WidgetHandle::new(log_out_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "log_out_button"
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
use duit::widgets::*;
use duit::*;
pub struct RegisterPage {
    pub error_text: WidgetHandle<Text>,
    pub username_input: WidgetHandle<TextInput>,
    pub email_input: WidgetHandle<TextInput>,
    pub password_input: WidgetHandle<TextInput>,
    pub verify_password_input: WidgetHandle<TextInput>,
    pub submit: WidgetHandle<Button>,
    pub login_link: WidgetHandle<Clickable>,
}
impl ::duit::InstanceHandle for RegisterPage {
    fn name() -> &'static str {
        "RegisterPage"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut error_text = None;
        let mut username_input = None;
        let mut email_input = None;
        let mut password_input = None;
        let mut verify_password_input = None;
        let mut submit = None;
        let mut login_link = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "error_text" => error_text = Some(widget),
                "username_input" => username_input = Some(widget),
                "email_input" => email_input = Some(widget),
                "password_input" => password_input = Some(widget),
                "verify_password_input" => verify_password_input = Some(widget),
                "submit" => submit = Some(widget),
                "login_link" => login_link = Some(widget),
                _ => {}
            }
        }
        Self {
            error_text: WidgetHandle::new(error_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "error_text"
                )
            })),
            username_input: WidgetHandle::new(username_input.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "username_input"
                )
            })),
            email_input: WidgetHandle::new(email_input.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "email_input"
                )
            })),
            password_input: WidgetHandle::new(password_input.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "password_input"
                )
            })),
            verify_password_input: WidgetHandle::new(verify_password_input.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "verify_password_input"
                )
            })),
            submit: WidgetHandle::new(submit.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "submit"
                )
            })),
            login_link: WidgetHandle::new(login_link.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "login_link"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct LoginPage {
    pub error_text: WidgetHandle<Text>,
    pub username_input: WidgetHandle<TextInput>,
    pub password_input: WidgetHandle<TextInput>,
    pub submit: WidgetHandle<Button>,
    pub register_link: WidgetHandle<Clickable>,
}
impl ::duit::InstanceHandle for LoginPage {
    fn name() -> &'static str {
        "LoginPage"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut error_text = None;
        let mut username_input = None;
        let mut password_input = None;
        let mut submit = None;
        let mut register_link = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "error_text" => error_text = Some(widget),
                "username_input" => username_input = Some(widget),
                "password_input" => password_input = Some(widget),
                "submit" => submit = Some(widget),
                "register_link" => register_link = Some(widget),
                _ => {}
            }
        }
        Self {
            error_text: WidgetHandle::new(error_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "error_text"
                )
            })),
            username_input: WidgetHandle::new(username_input.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "username_input"
                )
            })),
            password_input: WidgetHandle::new(password_input.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "password_input"
                )
            })),
            submit: WidgetHandle::new(submit.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "submit"
                )
            })),
            register_link: WidgetHandle::new(register_link.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "register_link"
                )
            })),
        }
    }
}
