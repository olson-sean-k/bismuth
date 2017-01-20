use math::{FScalar, UScalar};

mod cube;
mod generate;
mod primitive;
mod sphere;
mod tessellate;

pub use self::generate::{Conjoint, Indexed};
pub use self::primitive::{DecomposePolygon, DecomposePrimitive, Line, MapPrimitive, Polygon,
                          Triangle, Quad};
pub use self::tessellate::{Midpoint, TessellatePolygon, TessellateQuad};

pub type FCube = self::cube::FCube<FScalar>;
pub type UCube = self::cube::UCube<UScalar>;
pub type UVSphere = self::sphere::UVSphere<FScalar>;

#[cfg(test)]
mod tests {
    use super::*;
}
