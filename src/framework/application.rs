use event::React;
use render::MetaRenderer;
use super::context::{Context, RenderContextView, UpdateContextView};

pub trait Application<R>: React + Sized
    where R: MetaRenderer
{
    fn start(context: &mut Context<R>) -> Self;
    fn update<C>(&mut self, context: &mut C) where C: UpdateContextView;
    fn draw<C>(&mut self, context: &mut C) where C: RenderContextView<R>;
    fn stop(self);
}
