use glutin::Window;
use time::{Duration, PreciseTime};

use event::{Event, PollEvents, React};
use render::{AspectRatio, Context, GlutinContext, MetaContext};

pub trait TimeModel {
    fn tick(&mut self);
    fn react<F>(&mut self, f: F) where F: FnMut();
    fn update<F>(&mut self, f: F) where F: FnMut();
    fn draw<F>(&mut self, f: F) where F: FnMut();
    fn wait(&mut self);
}

pub struct FixedUpdate {
    now: PreciseTime,
    past: PreciseTime,
    lag: Duration,
    duration: Duration,
}

impl FixedUpdate {
    pub fn new(duration: Duration) -> Self {
        let now = PreciseTime::now();
        FixedUpdate {
            now: now,
            past: now,
            lag: Duration::zero(),
            duration: duration,
        }
    }
}

impl Default for FixedUpdate {
    fn default() -> Self {
        FixedUpdate::new(Duration::nanoseconds(16666667))
    }
}

impl TimeModel for FixedUpdate {
    fn tick(&mut self) {
        self.now = PreciseTime::now();
        self.lag = self.lag + (self.past.to(self.now));
        self.past = self.now;
    }

    fn react<F>(&mut self, mut f: F)
        where F: FnMut()
    {
        f();
    }

    fn update<F>(&mut self, mut f: F)
        where F: FnMut()
    {
        while self.lag >= self.duration {
            f();
            self.lag = self.lag - self.duration;
        }
    }

    fn draw<F>(&mut self, mut f: F)
        where F: FnMut()
    {
        f();
    }

    fn wait(&mut self) {}
}

pub struct Harness<C>
    where C: MetaContext
{
    context: Context<C>,
    dimensions: (u32, u32),
    aborting: bool,
}

impl Harness<GlutinContext> {
    pub fn from_glutin_window(window: Window) -> Self {
        Harness {
            dimensions: window.dimensions(),
            context: Context::from_glutin_window(window),
            aborting: false,
        }
    }
}

impl<C> Harness<C>
    where C: MetaContext
{
    pub fn start<A, T>(&mut self, mut timer: T)
        where A: Application<C>,
              T: TimeModel
    {
        let mut application = A::start(&mut self.context);
        'main: loop {
            if self.aborting {
                break 'main;
            }
            timer.tick();
            timer.react(|| {
                for event in self.context.window.poll_events() {
                    match event {
                        Event::Closed => {
                            self.abort();
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
            });
            timer.update(|| application.update(&mut self.context));
            timer.draw(|| {
                self.context.clear();
                application.draw(&mut self.context);
                self.context.flush().unwrap();
                self.dimensions = self.context.window.dimensions();
            });
            timer.wait();
        }
        application.stop();
    }

    pub fn abort(&mut self) {
        self.aborting = true;
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
