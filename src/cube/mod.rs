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

pub use self::geometry::*;
pub use self::space::*;
pub use self::tree::*;

#[cfg(test)]
mod tests {
    use super::*;
}
