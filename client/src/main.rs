#![allow(dead_code)]

use anyhow::Context as _;
use context::Context;
use simple_logger::SimpleLogger;

macro_rules! vars {
    ($(
        $name:ident => $val:expr
    ),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut map = ahash::AHashMap::new();
        $(
            map.insert(stringify!($name).to_owned(), $val.to_string());
        )*
        map
    }}
}

mod assets;
mod audio;
mod backend;
mod context;
mod event_loop;
mod game;
#[allow(warnings)]
mod generated;
mod registry;
mod state;
mod paths;
mod states;
mod ui;
mod volumes;
mod options;

use states::menu::MenuState;

extern crate fs_err as fs;

pub enum RootState {
    MainMenu(MenuState),
}

impl RootState {
    pub fn update(&mut self, cx: &mut Context) {
        match self {
            RootState::MainMenu(menu) => menu.update(cx),
        }
    }
}

fn init_logging() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("wgpu", log::LevelFilter::Warn) // wgpu spams Info level
        .init()
        .unwrap();
}

fn main() -> anyhow::Result<()> { 
    init_logging();

    // TEMP for testing
    std::env::set_current_dir("/Users/caelum/CLionProjects/riposte")?;

    let (mut context, event_loop) = Context::new()?;
    context.load_ui_specs().context("failed to load UI specs")?;
    context.load_assets().context("failed to load assets")?;

    let state = RootState::MainMenu(MenuState::new(&context));

    self::event_loop::run(event_loop, context, state);

    Ok(())
}
