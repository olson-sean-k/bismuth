use rand;

use math::{FScalar, FVector4};

mod camera;
mod context;
mod mesh;
pub mod pipeline;
mod texture;

pub use self::camera::{AspectRatio, Camera, Projection};
pub use self::context::Context;
pub use self::mesh::{MeshBuffer, ToMeshBuffer};
pub use self::pipeline::{Transform, Vertex};
pub use self::texture::Texture;

pub type Index = u32;
pub type Color = FVector4;

pub trait ColorExt {
    fn random() -> Self;
}

impl ColorExt for Color {
    fn random() -> Self {
        Color::new(rand::random::<FScalar>(),
                   rand::random::<FScalar>(),
                   rand::random::<FScalar>(),
                   1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
