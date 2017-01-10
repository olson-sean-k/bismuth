use gfx;
use glutin;
use nalgebra::{Isometry3, PerspectiveMatrix3, ToHomogeneous};
use rand;
use std::convert::AsRef;

use cube::{CubeRef, Spatial};
use math::{IntoSpace, FMatrix4, FPoint3, FScalar, FVector3, FVector4};
use mesh::{Conjoint, DecomposePolygon, DecomposePrimitive, Indexed, UCube};
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

impl From<Vertex> for RawVertex {
    fn from(vertex: Vertex) -> Self {
        RawVertex {
            position: *vertex.position.as_ref(),
            color: *vertex.color.as_ref(),
        }
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
            let color = random_color();

            let cube = UCube::with_unit_width();
            buffer.vertices.extend(cube.conjoint_points()
                .map(|point| leaf.geometry.map_unit_cube_point(&point))
                .map(|point| (point * width) + origin)
                .map(|point| RawVertex::from(Vertex::new(point, color))));
            buffer.indeces.extend(cube.indexed_polygons()
                .triangulate()
                .points()
                .map(|index| index as Index));
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

pub fn projection_from_window(window: &glutin::Window) -> FMatrix4 {
    let aspect = {
        let (width, height) = window.get_inner_size_pixels().unwrap();
        width as f32 / height as f32
    };
    PerspectiveMatrix3::new(aspect, 1.0, -1.0, 1.0).to_matrix()
}

pub fn look_at_cube<C>(cube: &C, from: &FPoint3) -> FMatrix4
    where C: Spatial
{
    Isometry3::look_at_rh(from,
                          &cube.partition().midpoint().into_space(),
                          &FVector3::new(0.0, 0.0, 1.0))
        .to_homogeneous()
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
