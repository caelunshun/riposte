use duit::Event;
use riposte_common::{assets::Handle, registry::Tech, CityId, PlayerId};
use winit::event::VirtualKeyCode;

use crate::{
    audio::SoundCategory,
    client::{self, Client},
    context::Context,
    game::{event::GameEvent, Game},
    renderer::GameRenderer,
    state::StateAttachment,
    states::game::prompts::{city_build::CityBuildPrompt, tech::TechPrompt},
    volumes,
};

use self::{
    city_screen::CityScreen,
    main_ui::MainUi,
    music::GameMusic,
    prompts::{research::ResearchPrompt, Prompts},
    sounds::GameSounds,
};

mod city_screen;
mod main_ui;
mod music;
mod prompts;
mod sounds;

enum Page {
    Main(MainUi),
    City(CityScreen),
}

impl Page {
    pub fn update(
        &mut self,
        cx: &mut Context,
        game: &Game,
        client: &mut Client<client::GameState>,
    ) {
        match self {
            Page::Main(ui) => ui.update(cx, game, client),
            Page::City(_) => {}
        }
    }

    pub fn handle_game_event(&mut self, cx: &mut Context, game: &Game, event: &GameEvent) {
        match self {
            Page::Main(ui) => ui.handle_game_event(cx, game, event),
            Page::City(ui) => ui.handle_game_event(cx, game, event),
        }
    }

    pub fn handle_event(
        &mut self,
        cx: &mut Context,
        game: &mut Game,
        client: &mut Client<client::GameState>,
        event: &Event,
    ) {
        match self {
            Page::Main(ui) => {
                if let Some(main_ui::Action::OpenCityScreen(city)) =
                    ui.handle_event(cx, game, client, event)
                {
                    *self = Page::City(CityScreen::new(cx, game, city));
                    game.current_city_screen = Some(city);
                }
            }
            Page::City(ui) => {
                if let Some(city_screen::Action::Close) = ui.handle_event(game, client, event) {
                    *self = Page::Main(MainUi::new(cx, game));
                    game.current_city_screen = None;
                }
            }
        }
    }
}

/// The game state, including game rendering and UI.
pub struct GameState {
    attachment: StateAttachment,

    game: Game,
    renderer: GameRenderer,
    client: Client<client::GameState>,

    page: Page,
    prompts: Prompts,
    music: GameMusic,
    sounds: GameSounds,
}

impl GameState {
    pub fn new(cx: &Context, client: Client<client::GameState>, game: Game) -> Self {
        let attachment = cx.state_manager().create_state();
        let renderer = GameRenderer::new(cx);

        let page = Page::Main(MainUi::new(cx, &game));

        if game.turn().get() == 0 {
            cx.popup_windows()
                .show_genesis_popup(&mut cx.ui_mut(), &game);
        }

        Self {
            attachment,
            game,
            renderer,
            client,
            page,
            prompts: Prompts::default(),
            music: GameMusic::new(cx),
            sounds: GameSounds::new(),
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<()> {
        self.client.handle_messages(cx, &mut self.game)?;
        self.prompts.update(cx, &self.game, &mut self.client);
        self.game.update(cx, &mut self.client);
        self.page.update(cx, &self.game, &mut self.client);
        self.renderer.render(&self.game, cx);

        self.music.update(cx, &self.game);

        while let Some(event) = self.game.next_event() {
            self.page.handle_game_event(cx, &self.game, &event);
            self.handle_game_event(cx, &event);
        }

        self.game.are_prompts_open = !self.prompts.is_empty();

        Ok(())
    }

    fn handle_game_event(&mut self, cx: &Context, event: &GameEvent) {
        self.sounds.handle_game_event(cx, &self.game, event);

        match event {
            GameEvent::CityUpdated { city } => {
                self.handle_city_updated(cx, *city);
            }
            GameEvent::PlayerUpdated { player } => {
                self.handle_player_updated(cx, *player);
            }
            GameEvent::TechUnlocked { tech } => self.handle_tech_unlocked(cx, tech),
            _ => {}
        }
    }

    fn handle_city_updated(&mut self, cx: &Context, city: CityId) {
        let city = self.game.city(city);
        if city.build_task().is_none() && city.owner() == self.game.the_player().id() {
            log::info!("Queueing build prompt for {}", city.name());
            self.prompts
                .push(CityBuildPrompt::new(&self.game, cx, city.id()));
        }
    }

    fn handle_player_updated(&mut self, cx: &Context, player: PlayerId) {
        if player == self.game.the_player().id() && self.game.turn().get() > 0 {
            if self.game.the_player().researching_tech().is_none() {
                self.prompts.push(ResearchPrompt::new(cx, &mut self.client));
            }
        }
    }

    fn handle_tech_unlocked(&mut self, cx: &Context, tech: &Handle<Tech>) {
        use heck::ToSnakeCase;

        self.prompts.push(TechPrompt::new(cx, tech.clone()));

        self.sounds.playing.push(cx.audio().play(
            "sound/event/tech_unlocked",
            SoundCategory::Effects,
            volumes::TECH,
        ));

        // Look for recording of tech quote
        let quote_recording = format!("narration/tech/{}", tech.name.to_snake_case());
        if cx.audio().contains_sound(&quote_recording) {
            self.sounds.playing.push(cx.audio().play(
                &quote_recording,
                SoundCategory::Effects,
                volumes::TECH,
            ));
        }
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        self.game.handle_event(cx, &mut self.client, event);
        self.page
            .handle_event(cx, &mut self.game, &mut self.client, event);

        if let Event::KeyPress {
            key: VirtualKeyCode::Return,
            ..
        } = event
        {
            if self.game.can_end_turn() {
                log::info!("Ending the turn");
                self.client.end_turn(&mut self.game);
            }
        }
    }
}
