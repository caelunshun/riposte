use duit::Event;
use winit::event::VirtualKeyCode;

use crate::{
    client::{self, Client},
    context::Context,
    game::{event::GameEvent, CityId, Game, PlayerId},
    renderer::GameRenderer,
    state::StateAttachment,
};

use self::{
    city_screen::CityScreen,
    main_ui::MainUi,
    prompts::{city_build::CityBuildPrompt, research::ResearchPrompt, Prompts},
};

mod city_screen;
mod main_ui;
mod prompts;

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
                    ui.handle_event(cx, game, event)
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
}

impl GameState {
    pub fn new(cx: &Context, client: Client<client::GameState>, game: Game) -> Self {
        let attachment = cx.state_manager().create_state();
        let renderer = GameRenderer::new(cx);

        let page = Page::Main(MainUi::new(cx, &game));

        Self {
            attachment,
            game,
            renderer,
            client,
            page,
            prompts: Prompts::default(),
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<()> {
        self.client.handle_messages(cx, &mut self.game)?;
        self.prompts.update(cx, &self.game, &mut self.client);
        self.game.update(cx, &mut self.client);
        self.page.update(cx, &self.game, &mut self.client);
        self.renderer.render(&self.game, cx);

        while let Some(event) = self.game.next_event() {
            self.page.handle_game_event(cx, &self.game, &event);
            self.handle_game_event(cx, &event);
        }

        self.game.are_prompts_open = !self.prompts.is_empty();

        Ok(())
    }

    fn handle_game_event(&mut self, cx: &Context, event: &GameEvent) {
        match event {
            GameEvent::CityUpdated { city } => {
                self.handle_city_updated(cx, *city);
            }
            GameEvent::PlayerUpdated { player } => {
                self.handle_player_updated(cx, *player);
            }
            _ => {}
        }
    }

    fn handle_city_updated(&mut self, cx: &Context, city: CityId) {
        let city = self.game.city(city);
        if city.build_task().is_none() && city.owner() == self.game.the_player().id() {
            log::info!("Queueing build prompt for {}", city.name());
            self.prompts.push(CityBuildPrompt::new(
                cx,
                &self.game,
                &mut self.client,
                city.id(),
            ));
        }
    }

    fn handle_player_updated(&mut self, cx: &Context, player: PlayerId) {
        if player == self.game.the_player().id() && self.game.turn() > 0 {
            if self.game.the_player().researching_tech().is_none() {
                self.prompts.push(ResearchPrompt::new(cx, &mut self.client));
            }
        }
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        self.game.handle_event(cx, &mut self.client, event);
        self.page
            .handle_event(cx, &mut self.game, &mut self.client, event);

        if let Event::KeyPress {
            key: VirtualKeyCode::Return,
        } = event
        {
            if self.game.can_end_turn() {
                log::info!("Ending the turn");
                self.client.end_turn(&mut self.game);
            }
        }
    }
}
