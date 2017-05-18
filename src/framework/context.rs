use glutin::Window;

use event::{Event, React};
use render::{AspectRatio, GlutinRenderer, MetaRenderer, Renderer};

pub trait WindowView: AspectRatio {
}

impl<T> WindowView for T
    where T: AspectRatio
{
}

pub trait State: Sized + React {
}

pub trait ContextView {
    type State;

    fn state(&self) -> &Self::State;
    fn window(&self) -> &WindowView;
}

pub trait UpdateContextView: ContextView {
    fn state_mut(&mut self) -> &mut Self::State;
}

pub trait RenderContextView<R>: ContextView
    where R: MetaRenderer
{
    fn renderer(&self) -> &Renderer<R>;
    fn renderer_mut(&mut self) -> &mut Renderer<R>;
}

pub struct Context<T, R>
    where T: State,
          R: MetaRenderer
{
    pub state: T,
    pub renderer: Renderer<R>,
}

impl<T, R> Context<T, R>
    where T: State,
          R: MetaRenderer
{
    pub fn new(state: T, renderer: Renderer<R>) -> Self {
        Context {
            state: state,
            renderer: renderer,
        }
    }
}

impl<T> Context<T, GlutinRenderer>
    where T: State
{
    pub fn from_glutin_window(state: T, window: Window) -> Self {
        Context::new(state, Renderer::from_glutin_window(window))
    }
}

impl<T, R> ContextView for Context<T, R>
    where T: State,
          R: MetaRenderer
{
    type State = T;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn window(&self) -> &WindowView {
        &self.renderer.window
    }
}

impl<T, R> UpdateContextView for Context<T, R>
    where T: State,
          R: MetaRenderer
{
    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }
}

impl<T, R> RenderContextView<R> for Context<T, R>
    where T: State,
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
    where T: State,
          R: MetaRenderer
{
    fn react(&mut self, event: &Event) {
        self.state.react(event);
        self.renderer.react(event);
    }
}
