use glutin::Window;

use event::{Event, PollEvents, React};
use render::{GlutinRenderer, MetaRenderer};
use super::application::Application;
use super::context::Context;

pub struct Harness<R>
    where R: MetaRenderer
{
    context: Context<R>,
}

impl Harness<GlutinRenderer> {
    pub fn from_glutin_window(window: Window) -> Self {
        Harness {
            context: Context::from_glutin_window(window),
        }
    }
}

impl<R> Harness<R>
    where R: MetaRenderer
{
    pub fn start<A>(&mut self)
        where A: Application
    {
        let mut application = A::start(&mut self.context);
        'main: loop {
            for event in self.context.renderer.window.poll_events() {
                match event {
                    Event::Closed => {
                        break 'main;
                    }
                    _ => {}
                }
                self.context.renderer.react(&event);
                application.react(&event);
            }
            if let Err(..) = application.update(&mut self.context) {
                break 'main;
            }
            if let Err(..) = application.render(&mut self.context) {
                break 'main;
            }
        }
        application.stop();
    }
}
