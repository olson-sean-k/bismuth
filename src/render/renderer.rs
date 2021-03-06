use failure::{Error, Fail};
use gfx::format::{DepthStencil, Rgba8, Srgba8};
use gfx::handle::{DepthStencilView, RenderTargetView};
use gfx::traits::FactoryExt;
use gfx::{CommandBuffer, Device, Encoder, Factory, PipelineState, Resources};
use gfx_device_gl;
use gfx_window_glutin;
use glutin::Window;
use plexus::buffer::MeshBuffer;

use event::{Event, PollEvents, React};
use framework::WindowView;
use render::camera::AspectRatio;
use render::pipeline::{self, Data, Meta, Transform, Vertex};
use render::texture::Texture;
use render::Index;

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub trait SwapBuffers {
    fn swap_buffers(&mut self) -> Result<(), Error>;
}

impl SwapBuffers for Window {
    fn swap_buffers(&mut self) -> Result<(), Error> {
        Window::swap_buffers(self).map_err(|error| error.into())
    }
}

pub trait UpdateFrameBufferView<R>
where
    R: Resources,
{
    fn update_frame_buffer_view(
        &self,
        color: &mut RenderTargetView<R, Rgba8>,
        depth: &mut DepthStencilView<R, DepthStencil>,
    );
}

impl UpdateFrameBufferView<gfx_device_gl::Resources> for Window {
    fn update_frame_buffer_view(
        &self,
        color: &mut RenderTargetView<gfx_device_gl::Resources, Rgba8>,
        depth: &mut DepthStencilView<gfx_device_gl::Resources, DepthStencil>,
    ) {
        gfx_window_glutin::update_views(self, color, depth);
    }
}

pub trait MetaRenderer {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    type Window: AspectRatio + PollEvents + SwapBuffers + UpdateFrameBufferView<Self::Resources> +
                 WindowView;
    type Resources: Resources;
    type Factory: Factory<Self::Resources>;
    type CommandBuffer: CommandBuffer<Self::Resources>;
    type Device: Device<Resources = Self::Resources, CommandBuffer = Self::CommandBuffer>;
}

pub struct GlutinRenderer {}

impl MetaRenderer for GlutinRenderer {
    type Window = Window;
    type Resources = gfx_device_gl::Resources;
    type Factory = gfx_device_gl::Factory;
    type CommandBuffer = gfx_device_gl::CommandBuffer;
    type Device = gfx_device_gl::Device;
}

pub struct Renderer<R>
where
    R: MetaRenderer,
{
    pub window: R::Window,
    pub factory: R::Factory,
    device: R::Device,
    encoder: Encoder<R::Resources, R::CommandBuffer>,
    state: PipelineState<R::Resources, Meta>,
    data: Data<R::Resources>,
}

impl Renderer<GlutinRenderer> {
    pub fn from_glutin_window(window: Window) -> Self {
        let (device, mut factory, color, depth) = gfx_window_glutin::init_existing(&window);
        let encoder = factory.create_command_buffer().into();
        Renderer::new(window, factory, device, encoder, color, depth)
    }
}

impl<R> Renderer<R>
where
    R: MetaRenderer,
{
    fn new(
        window: R::Window,
        mut factory: R::Factory,
        device: R::Device,
        encoder: Encoder<R::Resources, R::CommandBuffer>,
        color: RenderTargetView<R::Resources, Rgba8>,
        depth: DepthStencilView<R::Resources, DepthStencil>,
    ) -> Self {
        let state = factory
            .create_pipeline_simple(
                include_bytes!("../shader/cube.v.glsl"),
                include_bytes!("../shader/cube.f.glsl"),
                pipeline::new(),
            )
            .unwrap();
        let texture =
            Texture::<_, Srgba8>::from_file(&mut factory, "data/texture/default.png").unwrap();
        let data = Data {
            // Using an empty slice here causes an error.
            buffer: factory.create_vertex_buffer(&[Vertex::default()]),
            transform: factory.create_constant_buffer(1),
            camera: [[0.0; 4]; 4],
            model: [[0.0; 4]; 4],
            sampler: texture.to_pipeline_data(),
            color: color,
            depth: depth,
        };
        Renderer {
            window: window,
            factory: factory,
            device: device,
            encoder: encoder,
            state: state,
            data: data,
        }
    }

    pub fn set_transform(&mut self, transform: &Transform) -> Result<(), Error> {
        self.data.camera = transform.camera;
        self.data.model = transform.model;
        self.encoder
            .update_buffer(&self.data.transform, &[*transform], 0)
            .map_err(|error| error.context("failed to write transform buffer").into())
    }

    pub fn draw_mesh_buffer(&mut self, buffer: &MeshBuffer<Index, Vertex>) {
        let (buffer, slice) = self.factory
            .create_vertex_buffer_with_slice(buffer.as_vertex_slice(), buffer.as_index_slice());
        self.data.buffer = buffer;
        self.encoder.draw(&slice, &self.state, &self.data);
    }

    pub fn clear(&mut self) {
        self.encoder.clear(&self.data.color, CLEAR_COLOR);
        self.encoder.clear_depth(&self.data.depth, 1.0);
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().and_then(|_| {
            self.device.cleanup();
            Ok(())
        })
    }

    fn update_frame_buffer_view(&mut self) {
        self.window
            .update_frame_buffer_view(&mut self.data.color, &mut self.data.depth);
    }
}

impl<R> React for Renderer<R>
where
    R: MetaRenderer,
{
    fn react(&mut self, event: &Event) {
        if let Event::Resized(..) = *event {
            // TODO: Compare the dimensions from the event to the previous
            //       dimensions of the window and only update the frame buffer
            //       view if they differ. This should avoid spurious updates.
            self.update_frame_buffer_view();
        }
    }
}
