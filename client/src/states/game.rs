use duit::Event;

use crate::{client::{self, Client}, context::Context, game::{Game, event::GameEvent}, renderer::GameRenderer, state::StateAttachment};

use self::main_ui::MainUi;

mod main_ui;

enum Page {
    Main(MainUi),
}

impl Page {
    pub fn update(&mut self, cx: &mut Context, game: &Game, client: &mut Client<client::GameState>) {
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
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<()> {
        self.client.handle_messages(cx, &mut self.game)?;
        self.game.update(cx);
        self.page.update(cx, &self.game, &mut self.client);
        self.renderer.render(&self.game, cx);

        while let Some(event) = self.game.next_event() {
            self.page.handle_game_event(cx, &self.game, &event);
        }

        Ok(())
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        self.game.handle_event(cx, &mut self.client, event);
    }
}
