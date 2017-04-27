use glutin::Window;

use event::{Event, React};
use render::{AspectRatio, GlutinRenderer, MetaRenderer, Renderer};

pub trait ContextView {
    type Data: React;
    type Window: AspectRatio;

    fn data(&self) -> &Self::Data;

    // TODO: Consider using a trait object here instead. Using an associated
    //       type requires naming a particular type which may implement traits
    //       that should not be directly accessible to `update` (for example,
    //       `poll_events`).
    fn window(&self) -> &Self::Window;
}

pub trait UpdateContextView: ContextView {
    fn data_mut(&mut self) -> &mut Self::Data;
}

pub trait RenderContextView<R>: ContextView
    where R: MetaRenderer
{
    fn renderer(&self) -> &Renderer<R>;
    fn renderer_mut(&mut self) -> &mut Renderer<R>;
}

pub struct Context<T, R>
    where T: React,
          R: MetaRenderer
{
    pub data: T,
    pub renderer: Renderer<R>,
}

impl<T, R> Context<T, R>
    where T: React,
          R: MetaRenderer
{
    pub fn new(data: T, renderer: Renderer<R>) -> Self {
        Context {
            data: data,
            renderer: renderer,
        }
    }
}

impl<T> Context<T, GlutinRenderer>
    where T: React
{
    pub fn from_glutin_window(data: T, window: Window) -> Self {
        Context::new(data, Renderer::from_glutin_window(window))
    }
}

impl<T, R> ContextView for Context<T, R>
    where T: React,
          R: MetaRenderer
{
    type Data = T;
    type Window = R::Window;

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn window(&self) -> &Self::Window {
        &self.renderer.window
    }
}

impl<T, R> UpdateContextView for Context<T, R>
    where T: React,
          R: MetaRenderer
{
    fn data_mut(&mut self) -> &mut Self::Data {
        &mut self.data
    }
}

impl<T, R> RenderContextView<R> for Context<T, R>
    where T: React,
          R: MetaRenderer
{
    fn renderer(&self) -> &Renderer<R> {
        &self.renderer
    }

    fn renderer_mut(&mut self) -> &mut Renderer<R> {
        &mut self.renderer
    }
}

impl<T, R> React for Context<T, R>
    where T: React,
          R: MetaRenderer
{
    fn react(&mut self, event: &Event) {
        self.data.react(event);
        self.renderer.react(event);
    }
}
