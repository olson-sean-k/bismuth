//! This module defines graphics pipelines and types bound to those pipelines.
//!
//! Namely, this includes vertex and uniform buffer types.

use gfx;
use plexus::r32;
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

gfx_vertex_struct!{
    Vertex {
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

    fn ordered(&self) -> OrderedVertex {
        OrderedVertex {
            position: [
                r32::from(self.position[0]),
                r32::from(self.position[1]),
                r32::from(self.position[2]),
            ],
            uv: [r32::from(self.uv[0]), r32::from(self.uv[1])],
            color: [
                r32::from(self.color[0]),
                r32::from(self.color[1]),
                r32::from(self.color[2]),
                r32::from(self.color[3]),
            ],
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

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.ordered().hash(state);
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.ordered().eq(&other.ordered())
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct OrderedVertex {
    position: [r32; 3],
    uv: [r32; 2],
    color: [r32; 4],
}
