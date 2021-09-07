#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use context::Context;
use duit::Event;
use game::Game;
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
mod client;
mod context;
mod event_loop;
mod game;
#[allow(warnings)]
mod generated;
mod lobby;
mod options;
mod paths;
mod popups;
mod registry;
mod renderer;
mod server_bridge;
mod state;
mod states;
mod ui;
mod utils;
mod tooltips;
mod volumes;

use states::{game::GameState, lobby::GameLobbyState, menu::MenuState};

extern crate fs_err as fs;

pub enum Action {
    EnterSingleplayerLobby,
}

pub enum RootState {
    MainMenu(MenuState),
    Lobby(GameLobbyState),
    Game(GameState),
}

impl RootState {
    pub fn update(&mut self, cx: &mut Context) {
        match self {
            RootState::MainMenu(menu) => {
                if let Some(action) = menu.update(cx) {
                    match action {
                        Action::EnterSingleplayerLobby => {
                            match GameLobbyState::new_singleplayer(cx) {
                                Ok(l) => *self = RootState::Lobby(l),
                                Err(e) => cx.show_error_popup(&format!(
                                    "failed to create singleplayer game: {}",
                                    e
                                )),
                            }
                        }
                    }
                }
            }
            RootState::Lobby(lobby) => match lobby.update(cx) {
                Ok(Some(states::lobby::Action::EnterGame(game_data))) => {
                    let game =
                        match Game::from_initial_data(Arc::clone(cx.registry()), cx, game_data) {
                            Ok(g) => g,
                            Err(e) => {
                                cx.show_error_popup(&format!("failed to start game: {}", e));
                                *self = RootState::MainMenu(MenuState::new(cx));
                                return;
                            }
                        };
                    let client = lobby.client().to_game_state();
                    *self = RootState::Game(GameState::new(cx, client, game));
                }
                Ok(None) => {}
                Err(e) => {
                    cx.show_error_popup(&format!("disconnected from game: {}", e));
                    *self = RootState::MainMenu(MenuState::new(cx));
                }
            },
            RootState::Game(game) => {
                if let Err(e) = game.update(cx) {
                    cx.show_error_popup(&format!("disconnected from the game: {}", e));
                    *self = RootState::MainMenu(MenuState::new(cx));
                }
            }
        }
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        match self {
            RootState::Game(g) => g.handle_event(cx, event),
            _ => {}
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
