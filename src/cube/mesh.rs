use OptionExt;
use math::{IntoSpace, FPoint2, FPoint3, FScalar, FVector3, UScalar};
use mesh::{self, MapPoints, Points, Triangle, Triangulate};
use mesh::cube::Plane;
use render::{Color, Index, MeshBuffer, ToMeshBuffer, Vertex};
use super::space::{LogWidth, Spatial};
use super::tree::{BranchPayload, Cube, LeafPayload, Node, OrphanCube};

impl<'a, 'b> ToMeshBuffer for Cube<'a, &'b Node> {
    fn to_mesh_buffer(&self) -> MeshBuffer {
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
    fn to_mesh_buffer(&self) -> MeshBuffer {
        let mut buffer = MeshBuffer::new();
        if let Some(leaf) = self.as_leaf().and_if(|leaf| !leaf.geometry.is_empty()) {
            let origin: FVector3 = self.partition().origin().coords.into_space();
            let width = self.partition().width().exp() as FScalar;
            let ucube = mesh::cube::Cube::<UScalar>::with_unit_width();
            buffer.extend(
                ucube
                    .polygons()
                    .map_points(|point| leaf.geometry.map_unit_cube_point(&point))
                    .map_points(|point| (point * width) + origin)
                    .triangulate()
                    .zip(ucube.plane_polygons().triangulate())
                    .map(|(position, plane)| {
                        let color = Color::white();
                        Triangle::new(
                            Vertex::new(&position.a, &uv(plane.a, &position.a), &color),
                            Vertex::new(&position.b, &uv(plane.b, &position.b), &color),
                            Vertex::new(&position.c, &uv(plane.c, &position.c), &color),
                        )
                    })
                    .points(),
                ucube
                    .polygons()
                    .triangulate()
                    .points()
                    .enumerate()
                    .map(|(index, _)| index as Index),
            );
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
