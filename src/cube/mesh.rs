use plexus;
use plexus::buffer::MeshBuffer;
use plexus::generate::{self, HashIndexer};
use plexus::generate::cube::Plane;
use plexus::prelude::*;

use OptionExt;
use cube::space::{LogWidth, Spatial};
use cube::tree::{BranchPayload, Cube, LeafPayload, Node, OrphanCube};
use math::{FPoint2, FPoint3, FScalar, FVector3, IntoSpace, UPoint3, UScalar};
use render::{Color, Index, ToMeshBuffer, Vertex};

impl<'a, 'b> ToMeshBuffer for Cube<'a, &'b Node> {
    fn to_mesh_buffer(&self) -> MeshBuffer<Index, Vertex> {
        let mut buffer = MeshBuffer::new();
        for cube in self.iter() {
            buffer.append(&mut cube.as_orphan().to_mesh_buffer());
        }
        buffer
    }
}

impl<'a, L, B> ToMeshBuffer for OrphanCube<'a, L, B>
where
    L: AsRef<LeafPayload>,
    B: AsRef<BranchPayload>,
{
    fn to_mesh_buffer(&self) -> MeshBuffer<Index, Vertex> {
        let mut buffer = MeshBuffer::default();
        if let Some(leaf) = self.as_leaf().and_if(|leaf| !leaf.geometry.is_empty()) {
            let origin: FVector3 = self.partition().origin().coords.into_space();
            let width = self.partition().width().exp() as FScalar;

            // TODO: Do not use `MeshBuffer` for rendering. Because redundant
            //       vertices are used and the index buffer is fixed, instead
            //       expose slices and use a constant for the index buffer.
            let cube = generate::cube::Cube::default();
            buffer = MeshBuffer::<Index, Vertex>::from_raw_buffers(
                0..36u32,
                generate::zip_vertices((
                    cube.polygons_with_position()
                        .map_vertices(|position| -> FPoint3 { position.into() })
                        .map_vertices(|position| position + FVector3::new(0.5, 0.5, 0.5))
                        .map_vertices(|position| {
                            UPoint3::new(
                                position.x as UScalar,
                                position.y as UScalar,
                                position.z as UScalar,
                            )
                        })
                        .map_vertices(|position| leaf.geometry.map_unit_cube_point(&position))
                        .map_vertices(|position| (position * width) + origin),
                    cube.polygons_with_plane(),
                )).triangulate()
                    .map_vertices(|(position, plane)| {
                        Vertex::new(&position, &uv(plane, &position), &Color::white())
                    })
                    .vertices(),
            ).unwrap();
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
