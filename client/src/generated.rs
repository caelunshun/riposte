use crate::ui::flashing_button::FlashingButton; use crate::ui::turn_indicator::TurnIndicatorCircle;
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
pub struct ResearchBarWindow {
    pub research_progress: WidgetHandle<ProgressBar>,
    pub research_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for ResearchBarWindow {
    fn name() -> &'static str {
        "ResearchBarWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut research_progress = None;
        let mut research_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "research_progress" => research_progress = Some(widget),
                "research_text" => research_text = Some(widget),
                _ => {}
            }
        }
        Self {
            research_progress: WidgetHandle::new(research_progress.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "research_progress"
                )
            })),
            research_text: WidgetHandle::new(research_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "research_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityBuildPromptOption {
    pub clickable: WidgetHandle<Clickable>,
    pub option_text: WidgetHandle<Text>,
    pub tooltip_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityBuildPromptOption {
    fn name() -> &'static str {
        "CityBuildPromptOption"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut clickable = None;
        let mut option_text = None;
        let mut tooltip_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "clickable" => clickable = Some(widget),
                "option_text" => option_text = Some(widget),
                "tooltip_text" => tooltip_text = Some(widget),
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
            option_text: WidgetHandle::new(option_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "option_text"
                )
            })),
            tooltip_text: WidgetHandle::new(tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "tooltip_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct EconomyWindow {
    pub gold_text: WidgetHandle<Text>,
    pub expenses_text: WidgetHandle<Text>,
    pub revenue_text: WidgetHandle<Text>,
    pub beaker_percent_text: WidgetHandle<Text>,
    pub beaker_increment_button: WidgetHandle<Button>,
    pub beaker_decrement_button: WidgetHandle<Button>,
    pub beaker_output_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for EconomyWindow {
    fn name() -> &'static str {
        "EconomyWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut gold_text = None;
        let mut expenses_text = None;
        let mut revenue_text = None;
        let mut beaker_percent_text = None;
        let mut beaker_increment_button = None;
        let mut beaker_decrement_button = None;
        let mut beaker_output_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "gold_text" => gold_text = Some(widget),
                "expenses_text" => expenses_text = Some(widget),
                "revenue_text" => revenue_text = Some(widget),
                "beaker_percent_text" => beaker_percent_text = Some(widget),
                "beaker_increment_button" => beaker_increment_button = Some(widget),
                "beaker_decrement_button" => beaker_decrement_button = Some(widget),
                "beaker_output_text" => beaker_output_text = Some(widget),
                _ => {}
            }
        }
        Self {
            gold_text: WidgetHandle::new(gold_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "gold_text"
                )
            })),
            expenses_text: WidgetHandle::new(expenses_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "expenses_text"
                )
            })),
            revenue_text: WidgetHandle::new(revenue_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "revenue_text"
                )
            })),
            beaker_percent_text: WidgetHandle::new(beaker_percent_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "beaker_percent_text"
                )
            })),
            beaker_increment_button: WidgetHandle::new(beaker_increment_button.unwrap_or_else(
                || {
                    panic!(
                        "missing widget with ID '{}' (generated code not up to date)",
                        "beaker_increment_button"
                    )
                },
            )),
            beaker_decrement_button: WidgetHandle::new(beaker_decrement_button.unwrap_or_else(
                || {
                    panic!(
                        "missing widget with ID '{}' (generated code not up to date)",
                        "beaker_decrement_button"
                    )
                },
            )),
            beaker_output_text: WidgetHandle::new(beaker_output_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "beaker_output_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct UnitActionButton {
    pub the_button: WidgetHandle<FlashingButton>,
    pub the_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for UnitActionButton {
    fn name() -> &'static str {
        "UnitActionButton"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut the_button = None;
        let mut the_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "the_button" => the_button = Some(widget),
                "the_text" => the_text = Some(widget),
                _ => {}
            }
        }
        Self {
            the_button: WidgetHandle::new(the_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "the_button"
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
pub struct UnitActionBarWindow {
    pub actions: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for UnitActionBarWindow {
    fn name() -> &'static str {
        "UnitActionBarWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut actions = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "actions" => actions = Some(widget),
                _ => {}
            }
        }
        Self {
            actions: WidgetHandle::new(actions.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "actions"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct TurnIndicatorWindow {
    pub flag: WidgetHandle<Image>,
    pub turn_indicator: WidgetHandle<TurnIndicatorCircle>,
}
impl ::duit::InstanceHandle for TurnIndicatorWindow {
    fn name() -> &'static str {
        "TurnIndicatorWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut flag = None;
        let mut turn_indicator = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "flag" => flag = Some(widget),
                "turn_indicator" => turn_indicator = Some(widget),
                _ => {}
            }
        }
        Self {
            flag: WidgetHandle::new(flag.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "flag"
                )
            })),
            turn_indicator: WidgetHandle::new(turn_indicator.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "turn_indicator"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct ResearchPromptOption {
    pub clickable: WidgetHandle<Clickable>,
    pub option_text: WidgetHandle<Text>,
    pub tooltip_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for ResearchPromptOption {
    fn name() -> &'static str {
        "ResearchPromptOption"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut clickable = None;
        let mut option_text = None;
        let mut tooltip_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "clickable" => clickable = Some(widget),
                "option_text" => option_text = Some(widget),
                "tooltip_text" => tooltip_text = Some(widget),
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
            option_text: WidgetHandle::new(option_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "option_text"
                )
            })),
            tooltip_text: WidgetHandle::new(tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "tooltip_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityBuildPromptWindow {
    pub options_column: WidgetHandle<Flex>,
    pub question_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityBuildPromptWindow {
    fn name() -> &'static str {
        "CityBuildPromptWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut options_column = None;
        let mut question_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "options_column" => options_column = Some(widget),
                "question_text" => question_text = Some(widget),
                _ => {}
            }
        }
        Self {
            options_column: WidgetHandle::new(options_column.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "options_column"
                )
            })),
            question_text: WidgetHandle::new(question_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "question_text"
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
pub struct ResearchPromptWindow {
    pub options_column: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for ResearchPromptWindow {
    fn name() -> &'static str {
        "ResearchPromptWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut options_column = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "options_column" => options_column = Some(widget),
                _ => {}
            }
        }
        Self {
            options_column: WidgetHandle::new(options_column.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "options_column"
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
