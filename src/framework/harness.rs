use glutin::Window;

use event::{Event, PollEvents, React};
use framework::activity::{ActivityStack, BoxActivity};
use framework::context::{Context, State};
use render::{GlutinRenderer, MetaRenderer};

pub struct Harness<T, R>
where
    T: State,
    R: MetaRenderer,
{
    context: Context<T, R>,
}

impl<T> Harness<T, GlutinRenderer>
where
    T: State,
{
    pub fn from_glutin_window(state: T, window: Window) -> Self {
        Harness {
            context: Context::from_glutin_window(state, window),
        }
    }
}

impl<T, R> Harness<T, R>
where
    T: State,
    R: MetaRenderer,
{
    pub fn start<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Context<T, R>) -> BoxActivity<T, R>,
    {
        let mut stack = ActivityStack::new(f(&mut self.context));
        'main: loop {
            for event in self.context.renderer.window.poll_events() {
                if let Event::Closed = event {
                    break 'main;
                }
                self.context.react(&event);
                stack.react(&event);
            }
            match stack.update(&mut self.context) {
                Ok(false) | Err(..) => {
                    break 'main;
                }
                _ => {}
            }
            if let Err(..) = stack.render(&mut self.context) {
                break 'main;
            }
        }
    }
}
