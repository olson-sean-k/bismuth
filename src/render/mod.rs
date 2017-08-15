use plexus::buffer::conjoint::ConjointBuffer;
use rand;
use std::ops::{Deref, DerefMut};

use math::{FScalar, FVector4};

mod camera;
pub mod error;
pub mod pipeline;
mod renderer;
mod texture;

pub use self::camera::{AspectRatio, Camera, Projection};
pub use self::pipeline::{Transform, Vertex};
pub use self::renderer::{GlutinRenderer, MetaRenderer, Renderer};
pub use self::texture::Texture;

pub type Index = u32;

pub struct Color(FVector4);

impl Color {
    pub fn new(r: FScalar, g: FScalar, b: FScalar, a: FScalar) -> Self {
        Color(FVector4::new(r, g, b, a))
    }

    pub fn random() -> Self {
        Color::new(
            rand::random::<FScalar>(),
            rand::random::<FScalar>(),
            rand::random::<FScalar>(),
            1.0,
        )
    }

    pub fn white() -> Self {
        Color::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Color::new(0.0, 0.0, 0.0, 1.0)
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

pub trait ToConjointBuffer {
    fn to_conjoint_buffer(&self) -> ConjointBuffer<Index, Vertex>;
}
