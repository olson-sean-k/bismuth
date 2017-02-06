use gfx;
use rand;

use math::{FPoint3, FScalar, FVector4};

pub use self::pipeline::*;

pub type Index = u32;
pub type Color = FVector4;

pub trait ColorExt {
    fn random() -> Self;
}

impl ColorExt for Color {
    fn random() -> Self {
        Color::new(rand::random::<FScalar>(),
                   rand::random::<FScalar>(),
                   rand::random::<FScalar>(),
                   1.0)
    }
}

gfx_pipeline!{
    pipeline {
        buffer: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_transform",
        color: gfx::RenderTarget<gfx::format::Rgba8> = "f_target0",
        depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

gfx_vertex_struct!{
    Vertex {
        position: [f32; 3] = "a_position",
        color: [f32; 4] = "a_color",
    }
}

impl Vertex {
    pub fn new(position: &FPoint3, color: &Color) -> Self {
        Vertex {
            position: [position.x, position.y, position.z],
            color: [color.x, color.y, color.z, color.w],
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex::new(&FPoint3::origin(), &Color::new(0.0, 0.0, 0.0, 1.0))
    }
}
