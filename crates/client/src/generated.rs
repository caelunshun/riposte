use crate::ui::flashing_button::FlashingButton; use crate::ui::turn_indicator::TurnIndicatorCircle; use crate::ui::unit_indicator::UnitIndicator;
use duit::widgets::*;
use duit::*;
pub struct GameLobbyWindow {
    pub add_ai_slot_button: WidgetHandle<Button>,
    pub add_human_slot_button: WidgetHandle<Button>,
    pub slots_table: WidgetHandle<Table>,
    pub non_admin_group: WidgetHandle<Flex>,
    pub land_type: WidgetHandle<Text>,
    pub num_continents: WidgetHandle<Text>,
    pub map_size: WidgetHandle<Text>,
    pub admin_group: WidgetHandle<Flex>,
    pub land_type_picklist: WidgetHandle<PickList>,
    pub land_type_admin: WidgetHandle<Text>,
    pub num_continents_picklist: WidgetHandle<PickList>,
    pub num_continents_admin: WidgetHandle<Text>,
    pub map_size_picklist: WidgetHandle<PickList>,
    pub map_size_admin: WidgetHandle<Text>,
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
        let mut non_admin_group = None;
        let mut land_type = None;
        let mut num_continents = None;
        let mut map_size = None;
        let mut admin_group = None;
        let mut land_type_picklist = None;
        let mut land_type_admin = None;
        let mut num_continents_picklist = None;
        let mut num_continents_admin = None;
        let mut map_size_picklist = None;
        let mut map_size_admin = None;
        let mut start_game_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "add_ai_slot_button" => add_ai_slot_button = Some(widget),
                "add_human_slot_button" => add_human_slot_button = Some(widget),
                "slots_table" => slots_table = Some(widget),
                "non_admin_group" => non_admin_group = Some(widget),
                "land_type" => land_type = Some(widget),
                "num_continents" => num_continents = Some(widget),
                "map_size" => map_size = Some(widget),
                "admin_group" => admin_group = Some(widget),
                "land_type_picklist" => land_type_picklist = Some(widget),
                "land_type_admin" => land_type_admin = Some(widget),
                "num_continents_picklist" => num_continents_picklist = Some(widget),
                "num_continents_admin" => num_continents_admin = Some(widget),
                "map_size_picklist" => map_size_picklist = Some(widget),
                "map_size_admin" => map_size_admin = Some(widget),
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
            non_admin_group: WidgetHandle::new(non_admin_group.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "non_admin_group"
                )
            })),
            land_type: WidgetHandle::new(land_type.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "land_type"
                )
            })),
            num_continents: WidgetHandle::new(num_continents.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "num_continents"
                )
            })),
            map_size: WidgetHandle::new(map_size.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "map_size"
                )
            })),
            admin_group: WidgetHandle::new(admin_group.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "admin_group"
                )
            })),
            land_type_picklist: WidgetHandle::new(land_type_picklist.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "land_type_picklist"
                )
            })),
            land_type_admin: WidgetHandle::new(land_type_admin.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "land_type_admin"
                )
            })),
            num_continents_picklist: WidgetHandle::new(num_continents_picklist.unwrap_or_else(
                || {
                    panic!(
                        "missing widget with ID '{}' (generated code not up to date)",
                        "num_continents_picklist"
                    )
                },
            )),
            num_continents_admin: WidgetHandle::new(num_continents_admin.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "num_continents_admin"
                )
            })),
            map_size_picklist: WidgetHandle::new(map_size_picklist.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "map_size_picklist"
                )
            })),
            map_size_admin: WidgetHandle::new(map_size_admin.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "map_size_admin"
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
pub struct TechPopup {
    pub tech_name: WidgetHandle<Text>,
    pub quote_text: WidgetHandle<Text>,
    pub tooltip_text: WidgetHandle<Text>,
    pub close_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for TechPopup {
    fn name() -> &'static str {
        "TechPopup"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut tech_name = None;
        let mut quote_text = None;
        let mut tooltip_text = None;
        let mut close_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "tech_name" => tech_name = Some(widget),
                "quote_text" => quote_text = Some(widget),
                "tooltip_text" => tooltip_text = Some(widget),
                "close_button" => close_button = Some(widget),
                _ => {}
            }
        }
        Self {
            tech_name: WidgetHandle::new(tech_name.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "tech_name"
                )
            })),
            quote_text: WidgetHandle::new(quote_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "quote_text"
                )
            })),
            tooltip_text: WidgetHandle::new(tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "tooltip_text"
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
pub struct GenesisPopup {
    pub welcome_text: WidgetHandle<Text>,
    pub close_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for GenesisPopup {
    fn name() -> &'static str {
        "GenesisPopup"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut welcome_text = None;
        let mut close_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "welcome_text" => welcome_text = Some(widget),
                "close_button" => close_button = Some(widget),
                _ => {}
            }
        }
        Self {
            welcome_text: WidgetHandle::new(welcome_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "welcome_text"
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
pub struct SavesWindow {
    pub saves_table: WidgetHandle<Table>,
    pub back_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for SavesWindow {
    fn name() -> &'static str {
        "SavesWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut saves_table = None;
        let mut back_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "saves_table" => saves_table = Some(widget),
                "back_button" => back_button = Some(widget),
                _ => {}
            }
        }
        Self {
            saves_table: WidgetHandle::new(saves_table.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "saves_table"
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
pub struct ServerListWindow {
    pub games_table: WidgetHandle<Table>,
    pub back_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for ServerListWindow {
    fn name() -> &'static str {
        "ServerListWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut games_table = None;
        let mut back_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "games_table" => games_table = Some(widget),
                "back_button" => back_button = Some(widget),
                _ => {}
            }
        }
        Self {
            games_table: WidgetHandle::new(games_table.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "games_table"
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
use duit::widgets::*;
use duit::*;
pub struct UnitSelectionBarWindow {
    pub units: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for UnitSelectionBarWindow {
    fn name() -> &'static str {
        "UnitSelectionBarWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut units = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "units" => units = Some(widget),
                _ => {}
            }
        }
        Self {
            units: WidgetHandle::new(units.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "units"
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
pub struct ScoresWindow {
    pub scores_column: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for ScoresWindow {
    fn name() -> &'static str {
        "ScoresWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut scores_column = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "scores_column" => scores_column = Some(widget),
                _ => {}
            }
        }
        Self {
            scores_column: WidgetHandle::new(scores_column.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "scores_column"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct ResearchBarWindow {
    pub clickable: WidgetHandle<Clickable>,
    pub research_progress: WidgetHandle<ProgressBar>,
    pub research_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for ResearchBarWindow {
    fn name() -> &'static str {
        "ResearchBarWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut clickable = None;
        let mut research_progress = None;
        let mut research_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "clickable" => clickable = Some(widget),
                "research_progress" => research_progress = Some(widget),
                "research_text" => research_text = Some(widget),
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
pub struct UnitActionButton {
    pub the_button: WidgetHandle<FlashingButton>,
    pub the_text: WidgetHandle<Text>,
    pub tooltip_container: WidgetHandle<Container>,
    pub tooltip_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for UnitActionButton {
    fn name() -> &'static str {
        "UnitActionButton"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut the_button = None;
        let mut the_text = None;
        let mut tooltip_container = None;
        let mut tooltip_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "the_button" => the_button = Some(widget),
                "the_text" => the_text = Some(widget),
                "tooltip_container" => tooltip_container = Some(widget),
                "tooltip_text" => tooltip_text = Some(widget),
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
            tooltip_container: WidgetHandle::new(tooltip_container.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "tooltip_container"
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
pub struct CityResourcesEntry {
    pub resource_name: WidgetHandle<Text>,
    pub resource_output: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityResourcesEntry {
    fn name() -> &'static str {
        "CityResourcesEntry"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut resource_name = None;
        let mut resource_output = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "resource_name" => resource_name = Some(widget),
                "resource_output" => resource_output = Some(widget),
                _ => {}
            }
        }
        Self {
            resource_name: WidgetHandle::new(resource_name.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "resource_name"
                )
            })),
            resource_output: WidgetHandle::new(resource_output.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "resource_output"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityBuildingEntry {
    pub building_name: WidgetHandle<Text>,
    pub building_output: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityBuildingEntry {
    fn name() -> &'static str {
        "CityBuildingEntry"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut building_name = None;
        let mut building_output = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "building_name" => building_name = Some(widget),
                "building_output" => building_output = Some(widget),
                _ => {}
            }
        }
        Self {
            building_name: WidgetHandle::new(building_name.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "building_name"
                )
            })),
            building_output: WidgetHandle::new(building_output.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "building_output"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityInfoBarWindow {
    pub city_name: WidgetHandle<Text>,
    pub food_text: WidgetHandle<Text>,
    pub hammers_text: WidgetHandle<Text>,
    pub growth_progress_bar: WidgetHandle<ProgressBar>,
    pub growth_text: WidgetHandle<Text>,
    pub production_progress_bar: WidgetHandle<ProgressBar>,
    pub production_text: WidgetHandle<Text>,
    pub health_text: WidgetHandle<Text>,
    pub health_tooltip_text: WidgetHandle<Text>,
    pub health_sign_text: WidgetHandle<Text>,
    pub sick_text: WidgetHandle<Text>,
    pub sick_tooltip_text: WidgetHandle<Text>,
    pub happy_text: WidgetHandle<Text>,
    pub happy_tooltip_text: WidgetHandle<Text>,
    pub happy_sign_text: WidgetHandle<Text>,
    pub unhappy_text: WidgetHandle<Text>,
    pub unhappy_tooltip_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityInfoBarWindow {
    fn name() -> &'static str {
        "CityInfoBarWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut city_name = None;
        let mut food_text = None;
        let mut hammers_text = None;
        let mut growth_progress_bar = None;
        let mut growth_text = None;
        let mut production_progress_bar = None;
        let mut production_text = None;
        let mut health_text = None;
        let mut health_tooltip_text = None;
        let mut health_sign_text = None;
        let mut sick_text = None;
        let mut sick_tooltip_text = None;
        let mut happy_text = None;
        let mut happy_tooltip_text = None;
        let mut happy_sign_text = None;
        let mut unhappy_text = None;
        let mut unhappy_tooltip_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "city_name" => city_name = Some(widget),
                "food_text" => food_text = Some(widget),
                "hammers_text" => hammers_text = Some(widget),
                "growth_progress_bar" => growth_progress_bar = Some(widget),
                "growth_text" => growth_text = Some(widget),
                "production_progress_bar" => production_progress_bar = Some(widget),
                "production_text" => production_text = Some(widget),
                "health_text" => health_text = Some(widget),
                "health_tooltip_text" => health_tooltip_text = Some(widget),
                "health_sign_text" => health_sign_text = Some(widget),
                "sick_text" => sick_text = Some(widget),
                "sick_tooltip_text" => sick_tooltip_text = Some(widget),
                "happy_text" => happy_text = Some(widget),
                "happy_tooltip_text" => happy_tooltip_text = Some(widget),
                "happy_sign_text" => happy_sign_text = Some(widget),
                "unhappy_text" => unhappy_text = Some(widget),
                "unhappy_tooltip_text" => unhappy_tooltip_text = Some(widget),
                _ => {}
            }
        }
        Self {
            city_name: WidgetHandle::new(city_name.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "city_name"
                )
            })),
            food_text: WidgetHandle::new(food_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "food_text"
                )
            })),
            hammers_text: WidgetHandle::new(hammers_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "hammers_text"
                )
            })),
            growth_progress_bar: WidgetHandle::new(growth_progress_bar.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "growth_progress_bar"
                )
            })),
            growth_text: WidgetHandle::new(growth_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "growth_text"
                )
            })),
            production_progress_bar: WidgetHandle::new(production_progress_bar.unwrap_or_else(
                || {
                    panic!(
                        "missing widget with ID '{}' (generated code not up to date)",
                        "production_progress_bar"
                    )
                },
            )),
            production_text: WidgetHandle::new(production_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "production_text"
                )
            })),
            health_text: WidgetHandle::new(health_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "health_text"
                )
            })),
            health_tooltip_text: WidgetHandle::new(health_tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "health_tooltip_text"
                )
            })),
            health_sign_text: WidgetHandle::new(health_sign_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "health_sign_text"
                )
            })),
            sick_text: WidgetHandle::new(sick_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "sick_text"
                )
            })),
            sick_tooltip_text: WidgetHandle::new(sick_tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "sick_tooltip_text"
                )
            })),
            happy_text: WidgetHandle::new(happy_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "happy_text"
                )
            })),
            happy_tooltip_text: WidgetHandle::new(happy_tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "happy_tooltip_text"
                )
            })),
            happy_sign_text: WidgetHandle::new(happy_sign_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "happy_sign_text"
                )
            })),
            unhappy_text: WidgetHandle::new(unhappy_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "unhappy_text"
                )
            })),
            unhappy_tooltip_text: WidgetHandle::new(unhappy_tooltip_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "unhappy_tooltip_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityCultureWindow {
    pub culture_amount_text: WidgetHandle<Text>,
    pub culture_progress_bar: WidgetHandle<ProgressBar>,
    pub culture_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityCultureWindow {
    fn name() -> &'static str {
        "CityCultureWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut culture_amount_text = None;
        let mut culture_progress_bar = None;
        let mut culture_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "culture_amount_text" => culture_amount_text = Some(widget),
                "culture_progress_bar" => culture_progress_bar = Some(widget),
                "culture_text" => culture_text = Some(widget),
                _ => {}
            }
        }
        Self {
            culture_amount_text: WidgetHandle::new(culture_amount_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "culture_amount_text"
                )
            })),
            culture_progress_bar: WidgetHandle::new(culture_progress_bar.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "culture_progress_bar"
                )
            })),
            culture_text: WidgetHandle::new(culture_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "culture_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityResourcesWindow {
    pub resources_list: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for CityResourcesWindow {
    fn name() -> &'static str {
        "CityResourcesWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut resources_list = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "resources_list" => resources_list = Some(widget),
                _ => {}
            }
        }
        Self {
            resources_list: WidgetHandle::new(resources_list.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "resources_list"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityBuildingsWindow {
    pub buildings_list: WidgetHandle<Flex>,
}
impl ::duit::InstanceHandle for CityBuildingsWindow {
    fn name() -> &'static str {
        "CityBuildingsWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut buildings_list = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "buildings_list" => buildings_list = Some(widget),
                _ => {}
            }
        }
        Self {
            buildings_list: WidgetHandle::new(buildings_list.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "buildings_list"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct CityEconomyWindow {
    pub beaker_output_text: WidgetHandle<Text>,
    pub gold_output_text: WidgetHandle<Text>,
    pub maintenance_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for CityEconomyWindow {
    fn name() -> &'static str {
        "CityEconomyWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut beaker_output_text = None;
        let mut gold_output_text = None;
        let mut maintenance_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "beaker_output_text" => beaker_output_text = Some(widget),
                "gold_output_text" => gold_output_text = Some(widget),
                "maintenance_text" => maintenance_text = Some(widget),
                _ => {}
            }
        }
        Self {
            beaker_output_text: WidgetHandle::new(beaker_output_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "beaker_output_text"
                )
            })),
            gold_output_text: WidgetHandle::new(gold_output_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "gold_output_text"
                )
            })),
            maintenance_text: WidgetHandle::new(maintenance_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "maintenance_text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct TileTooltipWindow {
    pub root: WidgetHandle<Container>,
    pub tooltip_text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for TileTooltipWindow {
    fn name() -> &'static str {
        "TileTooltipWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut root = None;
        let mut tooltip_text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "root" => root = Some(widget),
                "tooltip_text" => tooltip_text = Some(widget),
                _ => {}
            }
        }
        Self {
            root: WidgetHandle::new(root.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "root"
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
pub struct InfoBarWindow {
    pub turn_text: WidgetHandle<Text>,
    pub save_game_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for InfoBarWindow {
    fn name() -> &'static str {
        "InfoBarWindow"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut turn_text = None;
        let mut save_game_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "turn_text" => turn_text = Some(widget),
                "save_game_button" => save_game_button = Some(widget),
                _ => {}
            }
        }
        Self {
            turn_text: WidgetHandle::new(turn_text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "turn_text"
                )
            })),
            save_game_button: WidgetHandle::new(save_game_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "save_game_button"
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
pub struct PlayerScore {
    pub clickable: WidgetHandle<Clickable>,
    pub text: WidgetHandle<Text>,
}
impl ::duit::InstanceHandle for PlayerScore {
    fn name() -> &'static str {
        "PlayerScore"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut clickable = None;
        let mut text = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "clickable" => clickable = Some(widget),
                "text" => text = Some(widget),
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
            text: WidgetHandle::new(text.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "text"
                )
            })),
        }
    }
}
use duit::widgets::*;
use duit::*;
pub struct UnitSelector {
    pub clickable: WidgetHandle<Clickable>,
    pub container: WidgetHandle<Container>,
    pub unit_head: WidgetHandle<Image>,
    pub indicators: WidgetHandle<UnitIndicator>,
}
impl ::duit::InstanceHandle for UnitSelector {
    fn name() -> &'static str {
        "UnitSelector"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut clickable = None;
        let mut container = None;
        let mut unit_head = None;
        let mut indicators = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "clickable" => clickable = Some(widget),
                "container" => container = Some(widget),
                "unit_head" => unit_head = Some(widget),
                "indicators" => indicators = Some(widget),
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
            container: WidgetHandle::new(container.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "container"
                )
            })),
            unit_head: WidgetHandle::new(unit_head.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "unit_head"
                )
            })),
            indicators: WidgetHandle::new(indicators.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "indicators"
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
