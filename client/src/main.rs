#![allow(dead_code)]

use anyhow::Context as _;
use context::Context;
use simple_logger::SimpleLogger;

mod assets;
mod audio;
mod context;
mod event_loop;
mod game;
#[allow(warnings)]
mod generated;
mod registry;
mod state;
mod states;
mod ui;
mod volumes;

use states::main_menu::MainMenuState;

extern crate fs_err as fs;

pub enum RootState {
    MainMenu(MainMenuState),
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

    let state = RootState::MainMenu(MainMenuState::new(&context));

    self::event_loop::run(event_loop, context, state);

    Ok(())
}
