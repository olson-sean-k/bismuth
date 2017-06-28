//! This module provides tools for generating meshes for simple shapes like
//! cubes and spheres. It uses an iterator-based interface that begins with a
//! unit shape and manipulates its constituent primitives like points, lines,
//! and polygons. All shapes provide position information and some can
//! additionally generate index, texture, and conjoint point information as
//! well.
//!
//! # Examples
//!
//! Generating position and index data for a scaled sphere mesh:
//!
//! ```
//! use bismuth::mesh::{ConjointPoints, IndexPolygons, Points, Triangulate};
//! use bismuth::mesh::sphere::UVSphere;
//!
//! let sphere = UVSphere::with_unit_radius(16, 16);
//! let positions: Vec<_> = sphere.conjoint_points().map(|point| point * 10.0).collect();
//! let indeces: Vec<_> = sphere.index_polygons().triangulate().points().collect();
//! ```

pub mod cube;
mod decompose;
mod generate;
mod primitive;
pub mod sphere;

pub use self::decompose::{Lines, Points, Subdivide, Tetrahedrons, Triangulate};
pub use self::generate::{ConjointPoints, IndexPolygons, Polygons, TexturePolygons};
pub use self::primitive::{Line, MapPoints, Polygon, Rotate, Triangle, Quad};

#[cfg(test)]
mod tests {
    use super::*;
}
