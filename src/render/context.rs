use gfx::{self, CommandBuffer, Device, Encoder, Factory, PipelineState, Resources};
use gfx::handle::{DepthStencilView, RenderTargetView};
use gfx::traits::FactoryExt;
use gfx_device_gl;
use gfx_window_glutin;
use glutin::{ContextError, Window};
use std::error::{self, Error};
use std::fmt;

use math::FMatrix4;
use super::mesh::MeshBuffer;
use super::pipeline::{self, Data, Meta, Vertex};

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

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
    fn new(window: W,
           mut factory: F,
           device: D,
           encoder: Encoder<R, B>,
           color: RenderTargetView<R, ColorFormat>,
           depth: DepthStencilView<R, DepthFormat>)
           -> Self {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let state = factory.create_pipeline_simple(include_bytes!("../shader/cube.v.glsl"),
                                                   include_bytes!("../shader/cube.f.glsl"),
                                                   pipeline::new())
            .unwrap();
        let data = Data {
            // Using an empty slice here causes an error.
            buffer: factory.create_vertex_buffer(&[Vertex::default()]),
            transform: [[0.0; 4]; 4],
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

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn set_transform(&mut self, transform: &FMatrix4) {
        let m = transform;
        self.data.transform = [[m[0],  m[1],  m[2],  m[3]],
                               [m[4],  m[5],  m[6],  m[7]],
                               [m[8],  m[9],  m[10], m[11]],
                               [m[12], m[13], m[14], m[15]]];
    }

    pub fn draw_mesh_buffer(&mut self, buffer: &MeshBuffer) {
        let (buffer, slice) = self.factory.create_vertex_buffer_with_slice(
            buffer.vertices(), buffer.indices());
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