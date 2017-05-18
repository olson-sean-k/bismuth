use std::convert::From;
use std::error;
use std::fmt;

use event::{Event, React};
use render::MetaRenderer;
use super::context::{Render, RenderContextView, State, Update, UpdateContextView};

pub enum Transition<T, R>
    where T: State,
          R: MetaRenderer
{
    None,
    Push(BoxActivity<T, R>),
    Pop,
    Abort,
}

impl<T, R> Transition<T, R>
    where T: State,
          R: MetaRenderer
{
    fn is_abort(&self) -> bool {
        match *self {
            Transition::Abort => true,
            _ => false,
        }
    }
}

// It would be natural to define `Activity` such that it must implement `Render`
// and `Update` as follows:
//
//   pub trait Activity<T, R>: React + Render<T, R, Error = ActivityError> +
//                             Update<T, Output = Transition<T, R>, Error = ActivityError>
//
// This doesn't work very well though. Rust does not respect or propogate
// constraints on associated types (as it does for input type parameters), so
// these types must be redeclared everywhere they are used. See the following
// issues:
//
//   https://github.com/rust-lang/rust/issues/23856
//   https://github.com/rust-lang/rust/issues/24010
//
// This redudancy also means that client code must repeat itself. A lot.
// Moreover, there is currently no way to define `Activity` as listed above
// because there is no way to disambiguate the associated types of `Render` and
// `Update` when redeclaring them elsewhere:
//
//   // Which `Error`?
//   pub type BoxActivity<T, R> = Activity<T, R, Error = ActivityError,
//                                         Output = Transition<T, R>>;
//
// Instead, `Activity` defines its own `render` and `update` functions and
// implements the `Render` and `Update` traits (for trait objects) by calling
// into those.

pub type BoxActivity<T, R> = Box<Activity<T, R>>;
pub type UpdateResult<T, R> = Result<Transition<T, R>, ActivityError>;
pub type RenderResult = Result<(), ActivityError>;

pub trait Activity<T, R>: React
    where T: State,
          R: MetaRenderer
{
    fn update(&mut self, context: &mut UpdateContextView<State = T>) -> UpdateResult<T, R>;
    fn render(&mut self, context: &mut RenderContextView<R, State = T>) -> RenderResult;
    fn pause(&mut self) {}
    fn resume(&mut self) {}
    fn stop(&mut self) {}
}

impl<T, R> Render<T, R> for Activity<T, R>
    where T: State,
          R: MetaRenderer
{
    type Error = ActivityError;

    fn render(&mut self, context: &mut RenderContextView<R, State = T>)
              -> Result<(), Self::Error>
    {
        <Self as Activity<T, R>>::render(self, context)
    }
}

impl<T, R> Update<T> for Activity<T, R>
    where T: State,
          R: MetaRenderer
{
    type Output = Transition<T, R>;
    type Error = ActivityError;

    fn update(&mut self, context: &mut UpdateContextView<State = T>)
              -> Result<Self::Output, Self::Error>
    {
        <Self as Activity<T, R>>::update(self, context)
    }
}

pub struct ActivityStack<T, R>
    where T: State,
          R: MetaRenderer
{
    stack: Vec<BoxActivity<T, R>>,
}

impl<T, R> ActivityStack<T, R>
    where T: State,
          R: MetaRenderer
{
    pub fn new(activity: BoxActivity<T, R>) -> Self {
        ActivityStack {
            stack: vec![activity],
        }
    }

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

impl<T, R> Drop for ActivityStack<T, R>
    where T: State,
          R: MetaRenderer
{
    fn drop(&mut self) {
        self.abort();
    }
}

impl<T, R> Update<T> for ActivityStack<T, R>
    where T: State,
          R: MetaRenderer
{
    type Output = bool;
    type Error = ActivityStackError;

    fn update(&mut self, context: &mut UpdateContextView<State = T>)
              -> Result<Self::Output, Self::Error>
    {
        let transition = self.peek_mut().map_or(
            Ok(Transition::Abort), |activity| activity.update(context))?;
        let signal = !transition.is_abort();
        match transition {
            Transition::Push(activity) => { self.push(activity); }
            Transition::Pop => { self.pop(); }
            Transition::Abort => { self.abort(); }
            _ => {}
        }
        Ok(signal)
    }
}

impl<T, R> Render<T, R> for ActivityStack<T, R>
    where T: State,
          R: MetaRenderer
{
    type Error = ActivityStackError;

    fn render(&mut self, context: &mut RenderContextView<R, State = T>)
              -> Result<(), Self::Error>
    {
        self.peek_mut().map_or(Ok(()), |activity| {
            activity.render(context).map_err(|error| error.into())
        })
    }
}

impl<T, R> React for ActivityStack<T, R>
    where T: State,
          R: MetaRenderer
{
    fn react(&mut self, event: &Event) {
        if let Some(activity) = self.peek_mut() {
            activity.react(event);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ActivityError;

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
pub struct ActivityStackError;

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

impl From<ActivityError> for ActivityStackError {
    fn from(_: ActivityError) -> Self {
        ActivityStackError
    }
}
