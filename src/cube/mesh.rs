use plexus;
use plexus::buffer::conjoint::ConjointBuffer;
use plexus::generate::{MapVertices, SpatialPolygons, Triangle, Triangulate};
use plexus::generate::cube::Plane;

use OptionExt;
use math::{FPoint2, FPoint3, FScalar, FVector3, IntoSpace, UPoint3, UScalar};
use render::{Color, Index, ToConjointBuffer, Vertex};
use super::space::{LogWidth, Spatial};
use super::tree::{BranchPayload, Cube, LeafPayload, Node, OrphanCube};

impl<'a, 'b> ToConjointBuffer for Cube<'a, &'b Node> {
    fn to_conjoint_buffer(&self) -> ConjointBuffer<Index, Vertex> {
        let mut buffer = ConjointBuffer::new();
        for cube in self.iter() {
            buffer.append(&mut cube.as_orphan().to_conjoint_buffer());
        }
        buffer
    }
}

impl<'a, L, B> ToConjointBuffer for OrphanCube<'a, L, B>
where
    L: AsRef<LeafPayload>,
    B: AsRef<BranchPayload>,
{
    fn to_conjoint_buffer(&self) -> ConjointBuffer<Index, Vertex> {
        let mut buffer = ConjointBuffer::new();
        if let Some(leaf) = self.as_leaf().and_if(|leaf| !leaf.geometry.is_empty()) {
            let origin: FVector3 = self.partition().origin().coords.into_space();
            let width = self.partition().width().exp() as FScalar;
            let cube = plexus::generate::cube::Cube::<UScalar>::with_unit_width();
            buffer.append::<Index, Vertex>(&mut cube.spatial_polygons()
                .map_vertices(|(x, y, z)| UPoint3::new(x, y, z))
                .map_vertices(|point| leaf.geometry.map_unit_cube_point(&point))
                .map_vertices(|point| (point * width) + origin)
                .triangulate()
                .zip(cube.planar_polygons().triangulate())
                .map(|(position, plane)| {
                    let color = Color::white();
                    Triangle::new(
                        Vertex::new(&position.a, &uv(plane.a, &position.a), &color),
                        Vertex::new(&position.b, &uv(plane.b, &position.b), &color),
                        Vertex::new(&position.c, &uv(plane.c, &position.c), &color),
                    )
                })
                .collect());
        }
        buffer
    }
}

fn uv(plane: Plane, point: &FPoint3) -> FPoint2 {
    fn map(x: FScalar) -> FScalar {
        x / LogWidth::unit().exp() as FScalar
    }
    match plane {
        Plane::XY => FPoint2::new(map(point.x), map(point.y)),
        Plane::NXY => FPoint2::new(-map(point.x), map(point.y)),
        Plane::ZY => FPoint2::new(map(point.z), map(point.y)),
        Plane::NZY => FPoint2::new(-map(point.z), map(point.y)),
        Plane::XZ => FPoint2::new(map(point.x), map(point.z)),
        Plane::XNZ => FPoint2::new(map(point.x), -map(point.z)),
    }
}
