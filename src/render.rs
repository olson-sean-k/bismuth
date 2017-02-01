use gfx;
use rand;
use std::convert::AsRef;

use cube::{CubeRef, Spatial};
use math::{IntoSpace, FPoint3, FScalar, FVector3, FVector4};
use mesh::{DecomposePolygon, DecomposePrimitive, MapPrimitive, Triangle, UCube};
use super::OptionExt;

pub type Index = u32;

gfx_pipeline!{
    pipeline {
        vertex_buffer: gfx::VertexBuffer<RawVertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_transform",
        output: gfx::RenderTarget<gfx::format::Rgba8> = "f_target0",
    }
}

gfx_vertex_struct!{
    RawVertex {
        position: [f32; 3] = "a_position",
        color: [f32; 4] = "a_color",
    }
}

impl RawVertex {
    fn new(position: &FPoint3, color: &FVector4) -> Self {
        RawVertex {
            position: *position.as_ref(),
            color: *color.as_ref(),
        }
    }
}

impl From<Vertex> for RawVertex {
    fn from(vertex: Vertex) -> Self {
        RawVertex::new(&vertex.position, &vertex.color)
    }
}

pub struct Vertex {
    pub position: FPoint3,
    pub color: FVector4,
}

impl Vertex {
    pub fn new(position: FPoint3, color: FVector4) -> Self {
        Vertex {
            position: position,
            color: color,
        }
    }
}

pub struct MeshBuffer {
    pub vertices: Vec<RawVertex>,
    pub indeces: Vec<Index>,
}

impl MeshBuffer {
    pub fn new() -> Self {
        MeshBuffer {
            vertices: vec![],
            indeces: vec![],
        }
    }

    pub fn extend(&mut self, buffer: &Self) {
        let offset = self.vertices.len();
        self.vertices.extend(buffer.vertices.iter());
        self.indeces.extend(buffer.indeces
            .iter()
            .map(|index| index + offset as Index));
    }
}

pub trait Mesh {
    fn mesh_buffer(&self) -> MeshBuffer;
}

impl<'a> Mesh for CubeRef<'a> {
    fn mesh_buffer(&self) -> MeshBuffer {
        let mut buffer = MeshBuffer::new();
        if let Some(leaf) = self.try_as_leaf().and_if(|leaf| !leaf.geometry.is_empty()) {
            let origin: FVector3 = self.partition().origin().to_vector().into_space();
            let width = self.partition().width().exp() as FScalar;

            buffer.vertices.extend(UCube::with_unit_width()
                .map_points(|point| leaf.geometry.map_unit_cube_point(&point))
                .map_points(|point| (point * width) + origin)
                .triangulate()
                .map(|triangle| {
                    let color = random_color();
                    Triangle::new(RawVertex::new(&triangle.a, &color),
                                  RawVertex::new(&triangle.b, &color),
                                  RawVertex::new(&triangle.c, &color))
                })
                .points());
            buffer.indeces.extend(UCube::with_unit_width()
                .triangulate()
                .points()
                .enumerate()
                .map(|(index, _)| index as Index));
        }
        buffer
    }
}

pub fn vertex_buffer_from_cube<R, F>(cube: &CubeRef,
                                     factory: &mut F)
                                     -> (gfx::handle::Buffer<R, RawVertex>, gfx::Slice<R>)
    where R: gfx::Resources,
          F: gfx::traits::FactoryExt<R>
{
    let mut buffer = MeshBuffer::new();
    for cube in cube.iter().filter(|cube| cube.is_leaf()) {
        buffer.extend(&cube.mesh_buffer());
    }
    factory.create_vertex_buffer_with_slice(buffer.vertices.as_slice(), buffer.indeces.as_slice())
}

fn random_color() -> FVector4 {
    FVector4::new(rand::random::<FScalar>(),
                  rand::random::<FScalar>(),
                  rand::random::<FScalar>(),
                  1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
}
