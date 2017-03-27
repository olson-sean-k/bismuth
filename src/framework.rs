use event::Reactor;
use render::{Context, GlutinContext, MetaContext};

pub struct Harness<A, C>
    where A: Application,
          C: MetaContext
{
    application: A,
    context: Context<C>,
}

impl<A> Harness<A, GlutinContext>
    where A: Application
{
    pub fn with_glutin_context(application: A) -> Self {
        panic!()
    }
}

pub trait Application {
    fn reactors(&mut self) -> &mut [&mut Reactor];
}
