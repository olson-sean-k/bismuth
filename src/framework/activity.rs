use std::error;
use std::fmt;

use event::{Event, React};
use render::MetaRenderer;
use super::application::Application;
use super::context::{Context, RenderContextView, UpdateContextView};

pub type BoxActivity<T, R> = Box<Activity<T, R>>;

pub enum Transition<T, R>
    where T: React,
          R: MetaRenderer
{
    None,
    Push(BoxActivity<T, R>),
    Pop,
    Abort,
}

pub trait Activity<T, R>: React
    where T: ActivityData<R>,
          R: MetaRenderer
{
    // TODO: What sort of `Result` (if any) should these functions yield?
    fn update(&mut self, context: &mut UpdateContextView<Data = T, Window = R::Window>)
              -> Transition<T, R>;
    fn render(&mut self, context: &mut RenderContextView<R, Data = T, Window = R::Window>);

    fn pause(&mut self) {}
    fn resume(&mut self) {}
    fn stop(&mut self) {}
}

pub trait ActivityData<R>: React + Sized
    where R: MetaRenderer
{
    // TODO: This seems like a hacky way to get an initial `Activity` onto the
    //       stack. Is there a better way?
    fn start(context: &mut Context<Self, R>) -> BoxActivity<Self, R>;
}

pub struct ActivityStack<T, R>
    where T: ActivityData<R>,
          R: MetaRenderer
{
    stack: Vec<BoxActivity<T, R>>,
}

impl<T, R> ActivityStack<T, R>
    where T: ActivityData<R>,
          R: MetaRenderer
{
    fn peek_mut(&mut self) -> Option<&mut Activity<T, R>> {
        if let Some(activity) = self.stack.last_mut() { // Cannot use `map`.
            Some(activity.as_mut())
        }
        else {
            None
        }
    }

    fn push(&mut self, activity: BoxActivity<T, R>) {
        if let Some(activity) = self.peek_mut() {
            activity.pause();
        }
        self.stack.push(activity);
    }

    fn pop(&mut self) -> bool {
        self.stack.pop().map(|mut activity| {
            activity.stop();
            if let Some(activity) = self.peek_mut() {
                activity.resume()
            }
        }).is_some()
    }

    fn abort(&mut self) {
        while self.pop() {}
    }
}

impl<T, R> Application<T, R> for ActivityStack<T, R>
    where T: ActivityData<R>,
          R: MetaRenderer
{
    type UpdateError = ActivityStackError;
    type RenderError = ActivityStackError;

    fn start(context: &mut Context<T, R>) -> Self {
        ActivityStack {
            stack: vec![T::start(context)],
        }
    }

    fn update<C>(&mut self, context: &mut C) -> Result<(), Self::UpdateError>
        where C: UpdateContextView<Data = T, Window = R::Window>
    {
        let transition = if let Some(activity) = self.peek_mut() {
            activity.update(context)
        }
        else {
            // TODO: At this point, we should stop the game loop.
            Transition::None
        };
        match transition {
            Transition::None => {}
            Transition::Push(activity) => { self.push(activity); }
            Transition::Pop => { self.pop(); }
            Transition::Abort => { self.abort(); }
        }
        Ok(())
    }

    fn render<C>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R, Data = T, Window = R::Window>
    {
        if let Some(activity) = self.peek_mut() {
            activity.render(context);
        }
        Ok(())
    }

    fn stop(mut self) {
        self.abort();
    }
}

impl<T, R> React for ActivityStack<T, R>
    where T: ActivityData<R>,
          R: MetaRenderer
{
    fn react(&mut self, event: &Event) {
        if let Some(activity) = self.peek_mut() {
            activity.react(event);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ActivityError {
}

impl fmt::Display for ActivityError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;

        write!(formatter, "{}", self.description())
    }
}

impl error::Error for ActivityError {
    fn description(&self) -> &str {
        ""
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ActivityStackError {
}

impl fmt::Display for ActivityStackError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;

        write!(formatter, "{}", self.description())
    }
}

impl error::Error for ActivityStackError {
    fn description(&self) -> &str {
        ""
    }
}
