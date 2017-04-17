use glutin::Window;

use render::{AspectRatio, GlutinRenderer, MeshBuffer, MetaRenderer, Renderer, RenderError,
             Transform};

pub trait ContextView {
    type Window: AspectRatio;

    fn window(&self) -> &Self::Window;
}

pub trait UpdateContextView: ContextView {
}

pub trait RenderContextView<R>: ContextView
    where R: MetaRenderer
{
    fn set_transform(&mut self, transform: &Transform) -> Result<(), RenderError>;
    fn draw_mesh_buffer(&mut self, buffer: &MeshBuffer);
}

pub struct Context<R>
    where R: MetaRenderer
{
    pub renderer: Renderer<R>,
}

impl<R> Context<R>
    where R: MetaRenderer
{
    pub fn from_renderer(renderer: Renderer<R>) -> Self {
        Context {
            renderer: renderer,
        }
    }
}

impl Context<GlutinRenderer> {
    pub fn from_glutin_window(window: Window) -> Self {
        Context::from_renderer(Renderer::from_glutin_window(window))
    }
}

impl<R> ContextView for Context<R>
    where R: MetaRenderer
{
    type Window = R::Window;

    fn window(&self) -> &Self::Window {
        &self.renderer.window
    }
}

impl<R> UpdateContextView for Context<R>
    where R: MetaRenderer
{
}

impl<R> RenderContextView<R> for Context<R>
    where R: MetaRenderer
{
    fn set_transform(&mut self, transform: &Transform) -> Result<(), RenderError> {
        self.renderer.set_transform(transform)
    }

    fn draw_mesh_buffer(&mut self, buffer: &MeshBuffer) {
        self.renderer.draw_mesh_buffer(buffer)
    }
}
