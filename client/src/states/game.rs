use duit::Event;

use crate::{
    client::{self, Client},
    context::Context,
    game::{event::GameEvent, CityId, Game},
    renderer::GameRenderer,
    state::StateAttachment,
};

use self::{
    main_ui::MainUi,
    prompts::{city_build::CityBuildPrompt, Prompts},
};

mod main_ui;
mod prompts;

enum Page {
    Main(MainUi),
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
        }
    }

    pub fn handle_game_event(&mut self, cx: &mut Context, game: &Game, event: &GameEvent) {
        match self {
            Page::Main(ui) => ui.handle_game_event(cx, game, event),
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
        self.game.update(cx);
        self.page.update(cx, &self.game, &mut self.client);
        self.prompts.update(cx, &self.game, &mut self.client);
        self.renderer.render(&self.game, cx);

        while let Some(event) = self.game.next_event() {
            self.page.handle_game_event(cx, &self.game, &event);
            self.handle_game_event(cx, &event);
        }

        Ok(())
    }

    fn handle_game_event(&mut self, cx: &Context, event: &GameEvent) {
        match event {
            GameEvent::CityUpdated { city } => {
                self.handle_city_updated(cx, *city);
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

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        self.game.handle_event(cx, &mut self.client, event);
    }
}
