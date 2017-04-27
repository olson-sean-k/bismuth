use std::error::Error;

use event::React;
use render::MetaRenderer;
use super::context::{Context, RenderContextView, UpdateContextView};

pub trait Application<T, R>: React + Sized
    where T: React,
          R: MetaRenderer
{
    type UpdateError: Error;
    type RenderError: Error;

    fn start(context: &mut Context<T, R>) -> Self;
    fn update<C>(&mut self, context: &mut C) -> Result<(), Self::UpdateError>
        where C: UpdateContextView<Data = T, Window = R::Window>;
    fn render<C>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R, Data = T, Window = R::Window>;
    fn stop(self);
}
