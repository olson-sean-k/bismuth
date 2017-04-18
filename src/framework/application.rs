use std::error::Error;

use event::React;
use render::MetaRenderer;
use super::context::{Context, RenderContextView, UpdateContextView};

pub trait Application<R>: React + Sized
    where R: MetaRenderer
{
    type UpdateError: Error;
    type RenderError: Error;

    fn start(context: &mut Context<R>) -> Self;
    fn update<C>(&mut self, context: &mut C) -> Result<(), Self::UpdateError>
        where C: UpdateContextView;
    fn render<C>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R>;
    fn stop(self);
}
