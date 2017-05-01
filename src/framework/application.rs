use std::error::Error;

use event::React;
use render::MetaRenderer;
use super::context::{Context, RenderContextView, UpdateContextView};

pub enum Execution {
    Continue,
    Abort
}

pub trait Application<T, R>: React + Sized
    where T: React,
          R: MetaRenderer
{
    type UpdateError: Error;
    type RenderError: Error;

    fn start(context: &mut Context<T, R>) -> Self;
    fn update<C>(&mut self, context: &mut C) -> Result<Execution, Self::UpdateError>
        where C: UpdateContextView<State = T, Window = R::Window>;
    fn render<C>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R, State = T, Window = R::Window>;
    fn stop(self);
}
