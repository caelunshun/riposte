use duit::Event;

use crate::{
    client::{self, Client},
    context::Context,
    game::Game,
    renderer::GameRenderer,
    state::StateAttachment,
};

/// The game state, including game rendering and UI.
pub struct GameState {
    attachment: StateAttachment,

    game: Game,
    renderer: GameRenderer,
    client: Client<client::GameState>,
}

impl GameState {
    pub fn new(cx: &Context, client: Client<client::GameState>, game: Game) -> Self {
        let attachment = cx.state_manager().create_state();
        let renderer = GameRenderer::new(cx);

        Self {
            attachment,
            game,
            renderer,
            client,
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> anyhow::Result<()> {
        self.client.handle_messages(&mut self.game)?;
        self.game.update(cx);
        self.renderer.render(&self.game, cx);
        Ok(())
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        self.game.handle_event(cx, &mut self.client, event);
    }
}
