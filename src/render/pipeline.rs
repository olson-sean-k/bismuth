//! This module defines graphics pipelines and types bound to those pipelines.
//!
//! Namely, this includes vertex and uniform buffer types.

use decorum;
use gfx;
use std::hash::{Hash, Hasher};

use math::{FMatrix4, FPoint2, FPoint3, Matrix4Ext};
use render::Color;

pub use self::pipeline::*;

gfx_pipeline!{
    pipeline {
        buffer: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "transform",
        camera: gfx::Global<[[f32; 4]; 4]> = "u_camera",
        model: gfx::Global<[[f32; 4]; 4]> = "u_model",
        sampler: gfx::TextureSampler<[f32; 4]> = "t_texture",
        color: gfx::RenderTarget<gfx::format::Rgba8> = "f_target0",
        depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

gfx_constant_struct!{
    Transform {
        camera: [[f32; 4]; 4] = "u_camera",
        model: [[f32; 4]; 4] = "u_model",
    }
}

impl Transform {
    pub fn new(camera: &FMatrix4, model: &FMatrix4) -> Self {
        Transform {
            camera: camera.to_array(),
            model: model.to_array(),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        let identity = FMatrix4::identity();
        Transform::new(&identity, &identity)
    }
}

gfx_vertex_struct_meta!{
    #[derive(PartialEq, PartialOrd)]
    vertex_struct_meta Vertex {
        position: [f32; 3] = "a_position",
        uv: [f32; 2] = "a_uv",
        color: [f32; 4] = "a_color",
    }
}

impl Vertex {
    pub fn new(position: &FPoint3, uv: &FPoint2, color: &Color) -> Self {
        Vertex {
            position: [position.x, position.y, position.z],
            uv: [uv.x, uv.y],
            color: [color.x, color.y, color.z, color.w],
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex::new(
            &FPoint3::origin(),
            &FPoint2::origin(),
            &Color::new(0.0, 0.0, 0.0, 1.0),
        )
    }
}

// TODO: This is incomplete and conversions are not a very efficient or
//       ergonomic way to implement `PartialEq`.
impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        decorum::hash_float_array(&self.position, state);
        decorum::hash_float_array(&self.uv, state);
        decorum::hash_float_array(&self.color, state);
    }
}
