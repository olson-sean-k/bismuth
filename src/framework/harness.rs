use glutin::Window;

use event::{Event, PollEvents, React};
use render::{GlutinRenderer, MetaRenderer};
use super::application::{Application, Execution};
use super::context::Context;

pub struct Harness<T, R>
    where T: React,
          R: MetaRenderer
{
    context: Context<T, R>,
}

impl<T> Harness<T, GlutinRenderer>
    where T: React
{
    pub fn from_glutin_window(data: T, window: Window) -> Self {
        Harness {
            context: Context::from_glutin_window(data, window),
        }
    }
}

impl<T, R> Harness<T, R>
    where T: React,
          R: MetaRenderer
{
    pub fn start<A>(&mut self)
        where A: Application<T, R>
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
                self.context.react(&event);
                application.react(&event);
            }
            match application.update(&mut self.context) {
                Ok(Execution::Abort) | Err(..) => {
                    break 'main;
                }
                _ => {}
            }
            if let Err(..) = application.render(&mut self.context) {
                break 'main;
            }
        }
        application.stop();
    }
}
