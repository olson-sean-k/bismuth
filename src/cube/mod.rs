//! This module provides an oct-tree of cubes, which are the basic building
//! blocks of Bismuth. A cube provides the following basic components:
//!
//!   1. Node data, which varies between branches and leaves.
//!   2. Node links, which refer to other nodes in the oct-tree.
//!   3. A spatial partition. This is calculated during traversals.
//!
//! Cubes bind this information together and act as a "recursive tree", where
//! any cube can be used to traverse its sub-tree of cubes.
//!
//! Leaf cubes provide geometric data that represents the shape and layout of
//! the game world.

mod geometry;
mod space;
mod tree;

use self::tree::{Cube, OrphanCube};

pub use self::geometry::{Edge, Geometry, MAX_EDGE_OFFSET, MIN_EDGE_OFFSET, Offset,
                         UNIT_CUBE_INDECES};
pub use self::space::{exp, AABB, Axis, Direction, MAX_WIDTH, MIN_WIDTH, LogWidth, Orientation,
                      Partition, Spatial};
pub use self::tree::{BranchNode, BranchPayload, LeafNode, LeafPayload, Node, OrphanNode, Root};

pub type CubeRef<'a> = Cube<'a, &'a Node>;
pub type CubeMut<'a> = Cube<'a, &'a mut Node>;
pub type OrphanCubeRef<'a> = OrphanCube<'a, &'a LeafPayload, &'a BranchPayload>;
pub type OrphanCubeMut<'a> = OrphanCube<'a, &'a mut LeafPayload, &'a mut BranchPayload>;

#[cfg(test)]
mod tests {
    use super::*;
}
