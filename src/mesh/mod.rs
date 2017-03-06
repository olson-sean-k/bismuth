use math::{FScalar, UScalar};

mod cube;
mod generate;
mod primitive;
mod sphere;
mod tessellate;

pub use self::generate::{Conjoint, Indexed, Textured};
pub use self::primitive::{DecomposePolygon, DecomposePrimitive, Line, MapPrimitive, Polygon,
                          Rotate, Triangle, Quad};
pub use self::tessellate::{TessellatePolygon, TessellateQuad};

pub type FCube = self::cube::RCube<FScalar>;
pub type UCube = self::cube::NCube<UScalar>;
pub type UVSphere = self::sphere::UVSphere<FScalar>;

#[cfg(test)]
mod tests {
    use super::*;
}
