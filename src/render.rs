extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate lazy_static;
extern crate nalgebra;
extern crate rand;

use nalgebra::ToHomogeneous;

use cube;
use cube::ComputedCube;
use math::{IntoDomain, RealSpace};

pub type Point3 = nalgebra::Point3<RealSpace>;
pub type Vector3 = nalgebra::Vector3<RealSpace>;
pub type Vector4 = nalgebra::Vector4<RealSpace>;
pub type Matrix4 = nalgebra::Matrix4<RealSpace>;
pub type Index = u32;

#[cfg_attr(rustfmt, rustfmt_skip)]
const UNIT_CUBE_INDECES: [Index; 36] = [
    0, 1, 2, 2, 3, 0,
    4, 5, 6, 6, 7, 4,
    6, 5, 2, 2, 1, 6,
    0, 3, 4, 4, 7, 0,
    5, 4, 3, 3, 2, 5,
    1, 0, 7, 7, 6, 1,
];
lazy_static! {
    static ref UNIT_CUBE_POINTS: [Point3; 8] = [
        // Back.
        Point3::new(0.0, 0.0, 1.0), // 0
        Point3::new(1.0, 0.0, 1.0), // 1
        Point3::new(1.0, 1.0, 1.0), // 2
        Point3::new(0.0, 1.0, 1.0), // 3
        // Front.
        Point3::new(0.0, 1.0, 0.0), // 4
        Point3::new(1.0, 1.0, 0.0), // 5
        Point3::new(1.0, 0.0, 0.0), // 6
        Point3::new(0.0, 0.0, 0.0), // 7
    ];
}

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

trait GeometricEdge {
    fn front_unit_transform(&self) -> f32;
    fn back_unit_transform(&self) -> f32;
}

impl GeometricEdge for cube::Edge {
    fn front_unit_transform(&self) -> f32 {
        ((self.front() - cube::MIN_EDGE) as f32) / ((cube::MAX_EDGE - cube::MIN_EDGE) as f32)
    }

    fn back_unit_transform(&self) -> f32 {
        let range = cube::MAX_EDGE - cube::MIN_EDGE;
        -((range - (self.back() - cube::MIN_EDGE)) as f32) / (range as f32)
    }
}

pub trait GeometricCube: cube::ComputedCube {
    fn points(&self) -> Vec<Point3>;
}

impl<T: cube::ComputedCube> GeometricCube for T {
    fn points(&self) -> Vec<Point3> {
        // TODO: Compute the points of this cube.
        unimplemented!()
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

struct Vertex {
    pub position: Point3,
    pub color: Vector4,
}

impl Vertex {
    pub fn new(position: Point3, color: Vector4) -> Self {
        Vertex {
            position: position,
            color: color,
        }
    }
}

pub fn vertex_buffer_from_cube<R, F>(cube: &cube::Traversal,
                                     factory: &mut F)
                                     -> (gfx::handle::Buffer<R, RawVertex>, gfx::Slice<R>)
    where R: gfx::Resources,
          F: gfx::traits::FactoryExt<R>
{
    let mut points = Vec::new();
    let mut indeces = Vec::new();
    for (i, cube) in cube.iter().filter(|cube| cube.is_leaf()).enumerate() {
        let width = cube.partition().width();
        let origin: Vector3 = cube.partition().origin().to_vector().into_domain();
        let color = Vector4::new(rand::random::<f32>(),
                                 rand::random::<f32>(),
                                 rand::random::<f32>(),
                                 1.0);
        points.extend(UNIT_CUBE_POINTS.iter()
            .map(|point| (point * cube::exp(width) as RealSpace) + origin)
            .map(|point| RawVertex::from(Vertex::new(point, color))));
        indeces.extend(UNIT_CUBE_INDECES.iter()
            .map(|j| ((UNIT_CUBE_POINTS.len() * i) as Index + *j)));
    }
    factory.create_vertex_buffer_with_slice(points.as_slice(), indeces.as_slice())
}

pub fn projection_from_window(window: &glutin::Window) -> Matrix4 {
    let aspect = {
        let (width, height) = window.get_inner_size_pixels().unwrap();
        width as f32 / height as f32
    };
    nalgebra::PerspectiveMatrix3::new(aspect, 1.0, -1.0, 1.0).to_matrix()
}

pub fn look_at_cube<C>(cube: &C, from: &Point3) -> Matrix4
    where C: cube::ComputedCube
{
    nalgebra::Isometry3::look_at_rh(from,
                                    &cube.partition().midpoint().into_domain(),
                                    &Vector3::new(0.0, 0.0, 1.0))
        .to_homogeneous()
}

#[cfg(test)]
mod tests {
    use super::*;
}
