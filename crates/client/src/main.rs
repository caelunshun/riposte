#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Context as _;
use client::Client;
use context::Context;
use duit::Event;
use game::Game;
use mimalloc::MiMalloc;
use riposte_common::bridge::{Bridge, ClientSide};
use server_bridge::ServerBridge;
use simple_logger::SimpleLogger;

#[global_allocator]
static ALLOCATOR: MiMalloc = MiMalloc;

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

mod asset_loaders;
mod audio;
mod backend;
mod client;
mod context;
mod event_loop;
mod fair_random;
mod game;
#[allow(warnings)]
mod generated;
mod options;
mod paths;
mod popups;
mod renderer;
mod saveload;
mod server_bridge;
mod state;
mod states;
// mod tooltips;
mod ui;
mod volumes;

use states::{lobby::GameLobbyState, menu::MenuState};

extern crate fs_err as fs;

pub enum Action {
    EnterSingleplayerLobby(Option<Vec<u8>>),
    EnterLobby(Bridge<ClientSide>),
}

pub enum RootState {
    MainMenu(MenuState),
    Lobby(GameLobbyState),
    //  Game(GameState),
}

impl RootState {
    pub fn update(&mut self, cx: &mut Context) {
        match self {
            RootState::MainMenu(menu) => {
                if let Some(action) = menu.update(cx) {
                    match action {
                        Action::EnterSingleplayerLobby(save) => {
                            match GameLobbyState::new_singleplayer(cx, save) {
                                Ok(l) => *self = RootState::Lobby(l),
                                Err(e) => cx.show_error_popup(&format!(
                                    "failed to create singleplayer game: {}",
                                    e
                                )),
                            }
                        }
                        Action::EnterLobby(bridge) => {
                            *self = RootState::Lobby(GameLobbyState::new(cx, Client::new(bridge)));
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
                    // *self = RootState::Game(GameState::new(cx, client, game));
                }
                Ok(None) => {}
                Err(e) => {
                    cx.show_error_popup(&format!("disconnected from game: {}", e));
                    *self = RootState::MainMenu(MenuState::new(cx));
                }
            },
            /*  RootState::Game(game) => {
                if let Err(e) = game.update(cx) {
                    cx.show_error_popup(&format!("disconnected from the game: {}", e));
                    *self = RootState::MainMenu(MenuState::new(cx));
                }
            }*/
        }
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        match self {
            // RootState::Game(g) => g.handle_event(cx, event),
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

    let (mut context, event_loop) = Context::new()?;
    context.load_ui_specs().context("failed to load UI specs")?;
    context.load_assets().context("failed to load assets")?;

    let state = RootState::MainMenu(MenuState::new(&context));

    self::event_loop::run(event_loop, context, state);

    Ok(())
}
