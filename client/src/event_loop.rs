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
                context.ui_mut().handle_window_event(
                    &mut *context.canvas_mut(),
                    &event,
                    context.window().scale_factor(),
                    window_logical_size,
                );

                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => context.resize(new_size),
                    _ => {}
                }

                context.handle_window_event(&event);

                let mut ui = context.ui_mut();
                if let Some(event) = ui
                    .convert_event(&event, context.window().scale_factor())
                {
                    drop(ui);
                    state.handle_event(&mut context, &event);
                }
            }

            _ => {}
        }
    });
}
