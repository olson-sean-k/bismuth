use glutin::Window;

use event::{Event, PollEvents, Reactor};
use render::{Context, GlutinContext, MetaContext};

pub struct Harness<A, C>
    where A: Application<C>,
          C: MetaContext
{
    application: A,
    context: Context<C>,
}

impl<A> Harness<A, GlutinContext>
    where A: Application<GlutinContext>
{
    pub fn from_glutin_window(application: A, window: Window) -> Self {
        Harness {
            application: application,
            context: Context::from_glutin_window(window),
        }
    }
}

impl<A, C> Harness<A, C>
    where A: Application<C>,
          C: MetaContext
{
    pub fn start(&mut self) {
        'main: loop {
            self.context.clear();
            for event in self.context.window.poll_events() {
                match event {
                    Event::Closed => {
                        break 'main;
                    }
                    Event::Resized(..) => {
                        self.context.update_frame_buffer_view();
                    }
                    _ => {}
                }
                self.application.react(&event);
            }
            self.application.update();
            self.application.draw(&mut self.context);
            self.context.flush().unwrap();
        }
        self.application.stop();
    }
}

pub trait Application<C>: Reactor
    where C: MetaContext
{
    fn update(&mut self);
    // TODO: Do not accept the entire `Context`. Maybe `Context` can emit a
    //       more limited type that can be used for rendering.
    fn draw(&mut self, context: &mut Context<C>);
    fn stop(&mut self) {}
}
