extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate lazy_static;
extern crate nalgebra;
extern crate rand;

use nalgebra::ToHomogeneous;

use cube;
use cube::{Cube, Node, Spatial};
use math::{IntoSpace, FMatrix4, FPoint3, FScalar, FVector3, FVector4};

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

struct Vertex {
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

pub fn vertex_buffer_from_cube<R, F>(cube: &Cube<&Node>,
                                     factory: &mut F)
                                     -> (gfx::handle::Buffer<R, RawVertex>, gfx::Slice<R>)
    where R: gfx::Resources,
          F: gfx::traits::FactoryExt<R>
{
    let mut points = Vec::new();
    let mut indeces = Vec::new();
    for (n, cube) in cube.iter().filter(|cube| cube.is_leaf()).enumerate() {
        if let Node::Leaf(ref leaf) = *cube {
            let width = cube.partition().width();
            let origin: FVector3 = cube.partition().origin().to_vector().into_space();
            let units = leaf.geometry.points();
            let color = random_color();
            // TODO: The ordering of the points must agree with the indeces.
            //       This seems to be broken (and can be seen when using more
            //       elaborate colors). Reversing the `Vec` seems to help, but
            //       this may still be broken, and using `rev` shouldn't be
            //       required of clients in any case.
            points.extend(units.iter().rev()
                .map(|point| (point * cube::exp(width) as FScalar) + origin)
                .map(|point| RawVertex::from(Vertex::new(point, color))));
            indeces.extend(leaf.geometry
                .indeces()
                .iter()
                .map(|index| ((units.len() * n) as Index + *index)));
        }
    }
    factory.create_vertex_buffer_with_slice(points.as_slice(), indeces.as_slice())
}

pub fn projection_from_window(window: &glutin::Window) -> FMatrix4 {
    let aspect = {
        let (width, height) = window.get_inner_size_pixels().unwrap();
        width as f32 / height as f32
    };
    nalgebra::PerspectiveMatrix3::new(aspect, 1.0, -1.0, 1.0).to_matrix()
}

pub fn look_at_cube<C>(cube: &C, from: &FPoint3) -> FMatrix4
    where C: Spatial
{
    nalgebra::Isometry3::look_at_rh(from,
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
