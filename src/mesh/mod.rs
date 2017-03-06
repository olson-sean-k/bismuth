use math::{FScalar, UScalar};

mod cube;
mod generate;
mod primitive;
mod sphere;
mod tessellate;

pub use self::generate::{Conjoint, Indexed, Textured};
pub use self::primitive::{DecomposePolygon, DecomposePrimitive, Line, MapPrimitive, Polygon,
                          RotatePrimitive, Triangle, Quad};
pub use self::tessellate::{TessellatePolygon, TessellateQuad};

pub type FCube = self::cube::Cube<FScalar>;
pub type UCube = self::cube::Cube<UScalar>;
pub type UVSphere = self::sphere::UVSphere<FScalar>;

#[cfg(test)]
mod tests {
    use super::*;
}
