use event::{Event, React};
use failure::Error;
use framework::context::{RenderContextView, State, UpdateContextView};
use framework::FrameworkError;
use render::MetaRenderer;

pub enum Transition<T, R>
where
    T: State,
    R: MetaRenderer,
{
    None,
    Push(BoxActivity<T, R>),
    Pop,
    Abort,
}

impl<T, R> Transition<T, R>
where
    T: State,
    R: MetaRenderer,
{
    fn is_abort(&self) -> bool {
        match *self {
            Transition::Abort => true,
            _ => false,
        }
    }
}

pub type BoxActivity<T, R> = Box<Activity<T, R>>;
pub type UpdateResult<T, R> = Result<Transition<T, R>, FrameworkError>;
pub type RenderResult = Result<(), FrameworkError>;

pub trait Activity<T, R>: React
where
    T: State,
    R: MetaRenderer,
{
    // TODO: It could be useful to extract `update` and `render` into generic
    //       traits of their own, but this would require associated types and
    //       there is currently no good way to bind those (until trait aliases
    //       land). Consider refactoring this once that is possible and clean.
    fn update(&mut self, context: &mut UpdateContextView<State = T>) -> UpdateResult<T, R>;
    fn render(&mut self, context: &mut RenderContextView<R, State = T>) -> RenderResult;
    fn suspend(&mut self) {}
    fn resume(&mut self) {}
    fn stop(&mut self) {}
}

pub struct ActivityStack<T, R>
where
    T: State,
    R: MetaRenderer,
{
    stack: Vec<BoxActivity<T, R>>,
}

impl<T, R> ActivityStack<T, R>
where
    T: State,
    R: MetaRenderer,
{
    pub fn new(activity: BoxActivity<T, R>) -> Self {
        ActivityStack {
            stack: vec![activity],
        }
    }

    pub fn update<C>(&mut self, context: &mut C) -> Result<bool, Error>
    where
        C: UpdateContextView<State = T>,
    {
        let transition = self.peek_mut()
            .map_or(Ok(Transition::Abort), |activity| activity.update(context))?;
        let signal = !transition.is_abort();
        match transition {
            Transition::Push(activity) => {
                self.push(activity);
            }
            Transition::Pop => {
                self.pop();
            }
            Transition::Abort => {
                self.abort();
            }
            _ => {}
        }
        Ok(signal)
    }

    pub fn render<C>(&mut self, context: &mut C) -> Result<(), Error>
    where
        C: RenderContextView<R, State = T>,
    {
        self.peek_mut().map_or(Ok(()), |activity| {
            activity.render(context).map_err(|error| error.into())
        })
    }

    fn peek_mut(&mut self) -> Option<&mut Activity<T, R>> {
        if let Some(activity) = self.stack.last_mut() {
            // Cannot use `map`.
            Some(activity.as_mut())
        }
        else {
            None
        }
    }

    fn push(&mut self, activity: BoxActivity<T, R>) {
        if let Some(activity) = self.peek_mut() {
            activity.suspend();
        }
        self.stack.push(activity);
    }

    fn pop(&mut self) -> bool {
        self.stack
            .pop()
            .map(|mut activity| {
                activity.stop();
                if let Some(activity) = self.peek_mut() {
                    activity.resume()
                }
            })
            .is_some()
    }

    fn abort(&mut self) {
        while self.pop() {}
    }
}

impl<T, R> Drop for ActivityStack<T, R>
where
    T: State,
    R: MetaRenderer,
{
    fn drop(&mut self) {
        self.abort();
    }
}

impl<T, R> React for ActivityStack<T, R>
where
    T: State,
    R: MetaRenderer,
{
    fn react(&mut self, event: &Event) {
        if let Some(activity) = self.peek_mut() {
            activity.react(event);
        }
    }
}
