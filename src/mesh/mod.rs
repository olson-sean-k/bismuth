pub mod cube;
mod decompose;
mod generate;
mod primitive;
pub mod sphere;

pub use self::decompose::{Lines, Points, Subdivide, Tetrahedrons, Triangulate};
pub use self::generate::{ConjointPoints, IndexPolygons, TexturePolygons};
pub use self::primitive::{Line, MapPoints, Polygon, Rotate, Triangle, Quad};

#[cfg(test)]
mod tests {
    use super::*;
}
