use OptionExt;
use cube::{CubeRef, Spatial};
use math::{IntoSpace, FScalar, FVector3};
use mesh::{DecomposePolygon, DecomposePrimitive, MapPrimitive, Triangle, UCube};
use super::pipeline::{Color, ColorExt, Index, Vertex};

pub struct MeshBuffer {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
}

impl MeshBuffer {
    pub fn new() -> Self {
        MeshBuffer {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn extend<V, I>(&mut self, vertices: V, indices: I)
        where V: IntoIterator<Item = Vertex>,
              I: IntoIterator<Item = Index>
    {
        self.vertices.extend(vertices);
        self.indices.extend(indices);
    }

    pub fn append(&mut self, buffer: &mut Self) {
        let offset = self.vertices.len();
        self.vertices.append(&mut buffer.vertices);
        self.indices.extend(buffer.indices.drain(..).map(|index| index + offset as Index));
    }

    pub fn vertices(&self) -> &[Vertex] {
        self.vertices.as_slice()
    }

    pub fn indices(&self) -> &[Index] {
        self.indices.as_slice()
    }
}

pub trait Mesh {
    fn mesh_buffer(&self) -> MeshBuffer;
}

impl<'a> Mesh for CubeRef<'a> {
    fn mesh_buffer(&self) -> MeshBuffer {
        let mut buffer = MeshBuffer::new();
        for cube in self.iter() {
            if let Some(leaf) = cube.try_as_leaf().and_if(|leaf| !leaf.geometry.is_empty()) {
                let origin: FVector3 = cube.partition().origin().coords.into_space();
                let width = cube.partition().width().exp() as FScalar;
                let mut local = MeshBuffer::new();
                local.extend(UCube::with_unit_width()
                                 .map_points(|point| leaf.geometry.map_unit_cube_point(&point))
                                 .map_points(|point| (point * width) + origin)
                                 .triangulate()
                                 .map(|triangle| {
                                     let color = Color::random();
                                     Triangle::new(Vertex::new(&triangle.a, &color),
                                                   Vertex::new(&triangle.b, &color),
                                                   Vertex::new(&triangle.c, &color))
                                 })
                                 .points(),
                             UCube::with_unit_width()
                                 .triangulate()
                                 .points()
                                 .enumerate()
                                 .map(|(index, _)| index as Index));
                buffer.append(&mut local);
            }
        }
        buffer
    }
}
