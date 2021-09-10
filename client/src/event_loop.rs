use duit::Vec2;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{context::Context, RootState};

pub fn run(event_loop: EventLoop<()>, mut context: Context, mut state: RootState) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => context.window().request_redraw(),

            Event::RedrawRequested(_) => {
                context.update();
                state.update(&mut context);
                context.render();
            }

            Event::WindowEvent { event, .. } => {
                let window_logical_size = context
                    .window()
                    .inner_size()
                    .to_logical(context.window().scale_factor());
                let window_logical_size =
                    Vec2::new(window_logical_size.width, window_logical_size.height);

                let duit_event = context
                    .ui_mut()
                    .convert_event(&event, context.window().scale_factor());

                if let Some(event) = &duit_event {
                    context.ui_mut().handle_window_event(
                        &mut *context.canvas_mut(),
                        event,
                        window_logical_size,
                    );
                }

                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => context.resize(new_size),
                    _ => {}
                }

                context.handle_window_event(&event);

                if let Some(event) = duit_event {
                    state.handle_event(&mut context, &event);
                }
            }

            _ => {}
        }
    });
}
