use glutin::Window;

use event::{Event, PollEvents, React};
use render::{AspectRatio, Context, GlutinContext, MetaContext};

pub struct Harness<C>
    where C: MetaContext
{
    context: Context<C>,
    dimensions: (u32, u32),
}

impl Harness<GlutinContext> {
    pub fn from_glutin_window(window: Window) -> Self {
        Harness {
            dimensions: window.dimensions(),
            context: Context::from_glutin_window(window),
        }
    }
}

impl<C> Harness<C>
    where C: MetaContext
{
    pub fn start<A>(&mut self)
        where A: Application<C>
    {
        let mut application = A::start(&mut self.context);
        'main: loop {
            self.context.clear();
            for event in self.context.window.poll_events() {
                match event {
                    Event::Closed => {
                        break 'main;
                    }
                    Event::Resized(width, height) => {
                        if self.dimensions.0 != width || self.dimensions.1 != height {
                            self.context.update_frame_buffer_view();
                        }
                    }
                    _ => {}
                }
                application.react(&event);
            }
            application.update(&mut self.context);
            application.draw(&mut self.context);
            self.dimensions = self.context.window.dimensions();
            self.context.flush().unwrap();
        }
        application.stop();
    }
}

// TODO: Many of `Application`s methods accept a rendering `Context`, but it
//       would be better to provide more targeted and limited parameters.
pub trait Application<C>: React + Sized
    where C: MetaContext
{
    fn start(context: &mut Context<C>) -> Self;
    fn update(&mut self, context: &mut Context<C>);
    fn draw(&mut self, context: &mut Context<C>);
    fn stop(self);
}
