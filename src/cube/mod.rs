//! This module provides an oct-tree of `Cube`s, which are the basic building
//! blocks of **Bismuth**.
//!
//! The tree is internally comprised of `Node`s. `BranchNode`s contain links to
//! sub-trees and `LeafNode`s contain geometric data associated with their
//! respective spatial partition. However, `Node`s do not contain any spatial
//! data (origin, partition, etc.); instead, this is calculated during
//! traversals by `Cube`, which acts as a recursive view over the `Node`s in the
//! tree and provides the primary interface for interacting with trees.
//!
//! Leaf `Cube`s provide the geometric and spatial data that together represent
//! the shape and layout of the game world.
//!
//! `Cube`s reference `Node`s in the tree, and abstract over the mutability of
//! those references. This module exposes the `CubeRef` and `CubeMut` type
//! definitions for immutable and mutable `Cube`s, respectively.
//!
//! Because `Cube`s reference `Node`s and `Node`s may reference other `Node`s
//! in a tree, orphan types are provided that only expose payloads and no links.
//! This is useful when collecting `Cube`s, because otherwise the references
//! into the rest of the tree would lead to aliasing. `OrphanCubeRef` and
//! `OrphanCubeMut` are the orphan analogues of `CubeRef` and `CubeMut`,
//! respectively. Orphans of course do not support indexing or traversal.
//!
//! `Root` can be used to create a new tree, and owns the root `Node`. `Root`s
//! expose `Cube`s to manipulate the tree.
//!
//! In the abstract, "cube" refers to the amalgamation of all the types used to
//! represent elements in a tree, which together form the complete notion of a
//! cube.
//!
//! # Examples
//!
//! Subdividing and iterating over the cubes in a tree:
//!
//! ```
//! use bismuth::cube::{LogWidth, Root, Spatial};
//!
//! let mut root = Root::new(LogWidth::max_value());
//! let _ = root.to_cube_mut().subdivide();
//! for cube in root.to_cube().iter() {
//!     println!("origin: {}; width: {}",
//!              cube.partition().origin(),
//!              cube.partition().width().to_inner());
//! }
//! ```

mod edit;
mod geometry;
mod space;
#[macro_use]
mod traverse;
mod tree;

use self::tree::{Cube, OrphanCube};

pub use self::edit::Cursor;
pub use self::geometry::{Edge, Geometry, Offset};
pub use self::space::{AABB, Axis, Direction, Intersects, LogWidth, Orientation, Partition,
                      RayCast, RayIntersection, Spatial};
pub use self::tree::{BranchNode, BranchPayload, LeafNode, LeafPayload, Node, OrphanNode, Root};

pub type CubeRef<'a> = Cube<'a, &'a Node>;
pub type CubeMut<'a> = Cube<'a, &'a mut Node>;
pub type OrphanCubeRef<'a> = OrphanCube<'a, &'a LeafPayload, &'a BranchPayload>;
pub type OrphanCubeMut<'a> = OrphanCube<'a, &'a mut LeafPayload, &'a mut BranchPayload>;

#[cfg(test)]
mod tests {
    use super::*;
}
