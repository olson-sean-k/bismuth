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
//! `Tree` can be used to create a new tree, and owns the root `Node`. `Tree`s
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
//! use bismuth::cube::{LogWidth, Spatial, Tree};
//!
//! let mut tree = Tree::new(LogWidth::max_value());
//! let _ = tree.as_cube_mut().subdivide();
//! for cube in tree.as_cube().iter() {
//!     println!(
//!         "origin: {}; width: {}",
//!         cube.partition().origin(),
//!         cube.partition().width().to_inner()
//!     );
//! }
//! ```

mod edit;
mod geometry;
mod mesh;
mod space;
#[macro_use]
mod traverse;
mod tree;

use self::tree::{Cube, OrphanCube};

pub use self::edit::Cursor;
pub use self::geometry::{Edge, Geometry, Offset};
pub use self::space::{Axis, Direction, Intersects, LogWidth, Orientation, Partition, RayCast,
                      RayIntersection, Spatial, AABB};
pub use self::tree::{BranchNode, BranchPayload, LeafNode, LeafPayload, Node, OrphanNode, Tree};

pub type CubeRef<'a, 'b> = Cube<'a, &'b Node>;
pub type CubeMut<'a, 'b> = Cube<'a, &'b mut Node>;
pub type OrphanCubeRef<'a, 'b> = OrphanCube<'a, &'b LeafPayload, &'b BranchPayload>;
pub type OrphanCubeMut<'a, 'b> = OrphanCube<'a, &'b mut LeafPayload, &'b mut BranchPayload>;

#[derive(Debug, Fail)]
pub enum CubeError {
    #[fail(display = "minimum width limit exceeded")]
    LimitExceeded,
    #[fail(display = "attempted to join leaf")]
    JoinLeaf,
    #[fail(display = "attempted to subdivide branch")]
    SubdivideBranch,
}
