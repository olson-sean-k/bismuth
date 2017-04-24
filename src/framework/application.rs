use std::error::Error;

use event::React;
use render::MetaRenderer;
use super::context::{Context, RenderContextView, UpdateContextView};

pub trait Application: React + Sized {
    type UpdateError: Error;
    type RenderError: Error;

    fn start<R>(context: &mut Context<R>) -> Self
        where R: MetaRenderer;
    fn update<C>(&mut self, context: &mut C) -> Result<(), Self::UpdateError>
        where C: UpdateContextView;
    fn render<C, R>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R>,
              R: MetaRenderer;
    fn stop(self);
}
