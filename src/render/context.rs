use gfx::{CommandBuffer, Device, Encoder, Factory, PipelineState, Resources};
use gfx::format::{DepthStencil, Rgba8};
use gfx::handle::{DepthStencilView, RenderTargetView, ShaderResourceView};
use gfx::texture::{AaMode, FilterMethod, Kind, SamplerInfo, WrapMode};
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_glutin;
use glutin::{ContextError, Window};
use image;
use std::error::{self, Error};
use std::fmt;

use super::mesh::MeshBuffer;
use super::pipeline::{self, Data, Meta, Transform, Vertex};

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub trait SwapBuffers {
    fn swap_buffers(&mut self) -> Result<(), RenderError>;
}

impl SwapBuffers for Window {
    fn swap_buffers(&mut self) -> Result<(), RenderError> {
        Window::swap_buffers(self).map_err(|error| {
            match error {
                ContextError::ContextLost => RenderError::ContextLost,
                _ => RenderError::Unknown,
            }
        })
    }
}

pub struct Context<W, R, F, B, D>
    where W: SwapBuffers,
          R: Resources,
          F: Factory<R>,
          B: CommandBuffer<R>,
          D: Device<Resources = R, CommandBuffer = B>
{
    pub window: W,
    factory: F,
    device: D,
    encoder: Encoder<R, B>,
    state: PipelineState<R, Meta>,
    data: Data<R>,
}

impl Context<Window,
             gfx_device_gl::Resources,
             gfx_device_gl::Factory,
             gfx_device_gl::CommandBuffer,
             gfx_device_gl::Device> {
    pub fn from_glutin_window(window: Window) -> Self {
        let (device, mut factory, color, depth) = gfx_window_glutin::init_existing(&window);
        let encoder = factory.create_command_buffer().into();
        Context::new(window, factory, device, encoder, color, depth)
    }
}

impl<W, R, F, B, D> Context<W, R, F, B, D>
    where W: SwapBuffers,
          R: Resources,
          F: Factory<R>,
          B: CommandBuffer<R>,
          D: Device<Resources = R, CommandBuffer = B>
{
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn new(window: W,
           mut factory: F,
           device: D,
           encoder: Encoder<R, B>,
           color: RenderTargetView<R, Rgba8>,
           depth: DepthStencilView<R, DepthStencil>)
           -> Self {
        let state = factory.create_pipeline_simple(include_bytes!("../shader/cube.v.glsl"),
                                                   include_bytes!("../shader/cube.f.glsl"),
                                                   pipeline::new())
            .unwrap();
        let sampler = factory.create_sampler(
            SamplerInfo::new(FilterMethod::Trilinear, WrapMode::Tile));
        let data = Data {
            // Using an empty slice here causes an error.
            buffer: factory.create_vertex_buffer(&[Vertex::default()]),
            transform: factory.create_constant_buffer(1),
            camera: [[0.0; 4]; 4],
            model: [[0.0; 4]; 4],
            sampler: (new_default_texture(&mut factory), sampler),
            color: color,
            depth: depth,
        };
        Context {
            window: window,
            factory: factory,
            device: device,
            encoder: encoder,
            state: state,
            data: data,
        }
    }

    pub fn set_transform(&mut self, transform: &Transform) -> Result<(), RenderError> {
        self.data.camera = transform.camera;
        self.data.model = transform.model;
        self.encoder.update_buffer(&self.data.transform, &[transform.clone()], 0).map_err(|_| {
            // TODO: Coerce and expose the `UpdateError`.
            RenderError::Unknown
        })
    }

    pub fn draw_mesh_buffer(&mut self, buffer: &MeshBuffer) {
        let (buffer, slice) = self.factory
            .create_vertex_buffer_with_slice(buffer.vertices(), buffer.indices());
        self.data.buffer = buffer;
        self.encoder.draw(&slice, &self.state, &self.data);
    }

    pub fn clear(&mut self) {
        self.encoder.clear(&self.data.color, CLEAR_COLOR);
        self.encoder.clear_depth(&self.data.depth, 1.0);
    }

    pub fn flush(&mut self) -> Result<(), RenderError> {
        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().and_then(|_| {
            self.device.cleanup();
            Ok(())
        })
    }
}

// TODO: Is there some way to generate an empty texture? This should not be
//       allowed to fail, and certainly shouldn't depend on data on disk.
fn new_default_texture<R, F>(factory: &mut F) -> ShaderResourceView<R, [f32; 4]>
    where R: Resources,
          F: Factory<R>
{
    let data = image::open("data/texture/default.png").unwrap().to_rgba();
    let (width, height) = data.dimensions();
    let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(
        Kind::D2(width as u16, height as u16, AaMode::Single),
        &[data.into_vec().as_slice()]).unwrap();
    view
}

#[derive(Debug)]
pub enum RenderError {
    ContextLost,
    Unknown,
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.description())
    }
}

impl error::Error for RenderError {
    fn description(&self) -> &str {
        match *self {
            RenderError::ContextLost => "rendering context lost",
            _ => "unknown rendering error",
        }
    }
}
