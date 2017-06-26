use super::Index;
use super::pipeline::Vertex;

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

pub trait ToMeshBuffer {
    fn to_mesh_buffer(&self) -> MeshBuffer;
}
