use duit::{
    widget,
    widgets::{Button, Text},
    WidgetHandle,
};

use crate::{
    context::Context,
    generated::SavesWindow,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

pub enum Action {
    Close,
    LoadGame(Vec<u8>),
}

struct Close;

struct LoadSave(usize);

pub struct SavesListState {
    state: StateAttachment,

    window: SavesWindow,
}

impl SavesListState {
    pub fn new(cx: &Context) -> Self {
        let state = cx.state_manager().create_state();

        let (window, _) = state.create_window::<SavesWindow, _>(FillScreen, Z_FOREGROUND);

        window.back_button.get_mut().on_click(|| Close);

        let mut table = window.saves_table.get_mut();
        table.add_row([
            ("created_at", widget(Text::new(text!("Created")))),
            ("turn", widget(Text::new(text!("Turn")))),
            ("load_button", widget(Text::new(text!("Actions")))),
        ]);
        for (i, save) in cx.saves().list_saves().enumerate() {
            let created_at = widget(Text::new(text!(
                "{}",
                humantime::format_rfc3339(save.created_at)
            )));
            let turn = widget(Text::new(text!("{}", save.turn)));

            let load_button = widget(Button::new());
            load_button
                .borrow_mut()
                .data_mut()
                .add_child(widget(Text::new(text!("Load"))));
            WidgetHandle::<Button>::new(load_button.clone())
                .get_mut()
                .on_click(move || LoadSave(i));

            table.add_row([
                ("created_at", created_at),
                ("turn", turn),
                ("load_button", load_button),
            ]);
        }
        drop(table);

        Self { state, window }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Action> {
        if cx.ui_mut().pop_message::<Close>().is_some() {
            return Some(Action::Close);
        }

        if let Some(LoadSave(index)) = cx.ui_mut().pop_message::<LoadSave>() {
            let saves = cx.saves();
            let save = saves.list_saves().skip(index).next().unwrap();
            let data = saves.load_save(cx, save);
            return Some(Action::LoadGame(data));
        }

        None
    }
}
