pub mod cube;
mod generate;
mod primitive;
pub mod sphere;
mod tessellate;

pub use self::generate::{ConjointPoints, IndexPolygons, TexturePolygons};
pub use self::primitive::{DecomposePolygon, DecomposePrimitive, Line, MapPrimitive, Polygon,
                          RotatePrimitive, Triangle, Quad};
pub use self::tessellate::{TessellatePolygon, TessellateQuad};

#[cfg(test)]
mod tests {
    use super::*;
}
