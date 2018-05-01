mod activity;
mod context;
mod harness;

pub use self::activity::{Activity, BoxActivity, RenderResult, Transition, UpdateResult};
pub use self::context::{Context, ContextView, RenderContextView, State, UpdateContextView,
                        WindowView};
pub use self::harness::Harness;

#[derive(Debug, Fail)]
pub enum FrameworkError {
    #[fail(display = "")]
    Activity,
    #[fail(display = "")]
    ActivityStack,
}
