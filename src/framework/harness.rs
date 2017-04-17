use glutin::Window;

use event::{Event, PollEvents};
use render::{AspectRatio, GlutinRenderer, MetaRenderer};
use super::application::Application;
use super::context::Context;

pub struct Harness<R>
    where R: MetaRenderer
{
    context: Context<R>,
    dimensions: (u32, u32),
}

impl Harness<GlutinRenderer> {
    pub fn from_glutin_window(window: Window) -> Self {
        Harness {
            dimensions: window.dimensions(),
            context: Context::from_glutin_window(window),
        }
    }
}

impl<R> Harness<R>
    where R: MetaRenderer
{
    pub fn start<A>(&mut self)
        where A: Application<R>
    {
        let mut application = A::start(&mut self.context);
        'main: loop {
            self.context.renderer.clear();
            for event in self.context.renderer.window.poll_events() {
                match event {
                    Event::Closed => {
                        break 'main;
                    }
                    Event::Resized(width, height) => {
                        if self.dimensions.0 != width || self.dimensions.1 != height {
                            self.context.renderer.update_frame_buffer_view();
                        }
                    }
                    _ => {}
                }
                application.react(&event);
            }
            application.update(&mut self.context);
            application.draw(&mut self.context);
            self.dimensions = self.context.renderer.window.dimensions();
            self.context.renderer.flush().unwrap();
        }
        application.stop();
    }
}
