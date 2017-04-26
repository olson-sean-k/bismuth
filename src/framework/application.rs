use std::error::Error;

use event::React;
use render::MetaRenderer;
use super::context::{Context, RenderContextView, UpdateContextView};

pub trait Application: React + Sized {
    type Data: React;

    type UpdateError: Error;
    type RenderError: Error;

    fn start<R>(context: &mut Context<Self::Data, R>) -> Self
        where R: MetaRenderer;
    fn update<C>(&mut self, context: &mut C) -> Result<(), Self::UpdateError>
        where C: UpdateContextView<Data = Self::Data>;
    fn render<C, R>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R, Data = Self::Data>,
              R: MetaRenderer;
    fn stop(self);
}
