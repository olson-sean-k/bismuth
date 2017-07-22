use std::hash::Hash;
use std::iter::FromIterator;

use mesh::{HashIndexer, IndexPrimitives, IntoPoints, Primitive};
use super::Index;
use super::pipeline::Vertex;

pub struct MeshBuffer {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
}

impl MeshBuffer {
    pub fn new() -> Self {
        MeshBuffer::default()
    }

    pub fn extend<V, I>(&mut self, vertices: V, indices: I)
    where
        V: IntoIterator<Item = Vertex>,
        I: IntoIterator<Item = Index>,
    {
        self.vertices.extend(vertices);
        self.indices.extend(indices);
    }

    pub fn append(&mut self, buffer: &mut Self) {
        let offset = self.vertices.len();
        self.vertices.append(&mut buffer.vertices);
        self.indices.extend(
            buffer
                .indices
                .drain(..)
                .map(|index| index + offset as Index),
        );
    }

    pub fn vertices(&self) -> &[Vertex] {
        self.vertices.as_slice()
    }

    pub fn indices(&self) -> &[Index] {
        self.indices.as_slice()
    }
}

impl Default for MeshBuffer {
    fn default() -> Self {
        MeshBuffer {
            vertices: vec![],
            indices: vec![],
        }
    }
}

// This allows for streams of polygons containing `Vertex`s to be `collect`ed
// into a `MeshBuffer`. This is a bit dubious; the high cost and complexity is
// hidden behind an innocuous `collect` invocation.
impl<T> FromIterator<T> for MeshBuffer
where
    T: IntoPoints + Primitive,
    T::Point: Eq + Hash + Into<Vertex>,
{
    fn from_iter<I>(input: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut buffer = MeshBuffer::new();
        let (indeces, points) = input.into_iter().index_primitives(HashIndexer::default());
        buffer.extend(
            points.into_iter().map(|point| point.into()),
            indeces.into_iter().map(|index| index as Index),
        );
        buffer
    }
}

pub trait ToMeshBuffer {
    fn to_mesh_buffer(&self) -> MeshBuffer;
}
