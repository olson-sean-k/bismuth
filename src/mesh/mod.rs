use math::{FScalar, UScalar};

mod cube;
mod generate;
mod primitive;

pub use self::generate::{Conjoint, Indexed};
pub use self::primitive::{DecomposePolygon, DecomposePrimitive, Line, Map, Polygon, Triangle, Quad};

pub type FCube = self::cube::FCube<FScalar>;
pub type UCube = self::cube::UCube<UScalar>;

#[cfg(test)]
mod tests {
    use super::*;
}
