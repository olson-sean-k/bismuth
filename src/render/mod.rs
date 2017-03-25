use rand;
use std::ops::{Deref, DerefMut};

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

pub struct Color(FVector4);

impl Color {
    pub fn new(r: FScalar, g: FScalar, b: FScalar, a: FScalar) -> Self {
        Color(FVector4::new(r, g, b, a))
    }

    pub fn random() -> Self {
        Color::new(rand::random::<FScalar>(),
                   rand::random::<FScalar>(),
                   rand::random::<FScalar>(),
                   1.0)
    }
}

impl Deref for Color {
    type Target = FVector4;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
