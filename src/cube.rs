//! This module provides an oct-tree of cubes. Each leaf cube describes a
//! spatial partition, its properties, and a deformable geometry.

extern crate nalgebra;
extern crate num;

use ::nalgebra::Origin;
use ::num::{One, Zero}; // TODO: `use ::std::num::{One, Zero};`.
use ::std::error;
use ::std::error::Error;
use ::std::fmt;
use ::std::mem;
use ::std::ops;

use math::{Clamp, Mask, DiscreteSpace};
use resource::ResourceId;

pub const MAX_WIDTH: RootWidth = 32;
pub const MIN_WIDTH: RootWidth = 4;

pub type Point3 = nalgebra::Point3<DiscreteSpace>;
pub type Vector3 = nalgebra::Vector3<DiscreteSpace>;
pub type RootWidth = u8; // TODO: https://github.com/rust-lang/rfcs/issues/671

impl Clamp<RootWidth> for RootWidth {
    fn clamp(&self, min: RootWidth, max: RootWidth) -> Self {
        nalgebra::clamp(*self, min, max)
    }
}

#[derive(Clone)]
pub struct Partition {
    origin: Point3,
    width: RootWidth,
}

impl Partition {
    fn at_point(point: &Point3, width: RootWidth) -> Self {
        Partition {
            origin: point.mask(!DiscreteSpace::zero() << width),
            width: width,
        }
    }

    fn at_index(&self, index: usize) -> Option<Self> {
        if self.width > MIN_WIDTH {
            let width = self.width - 1;
            Some(Partition {
                origin: self.origin + vector_at_index(index, width),
                width: width,
            })
        } else {
            None
        }
    }

    pub fn origin(&self) -> &Point3 {
        &self.origin
    }

    pub fn width(&self) -> RootWidth {
        self.width
    }

    pub fn midpoint(&self) -> Point3 {
        let m = exp(self.width - 1);
        Point3::new(m, m, m)
    }
}

pub trait ComputedCube: ops::Deref<Target = Cube> {
    fn partition(&self) -> &Partition;

    fn depth(&self) -> u8;
}

#[derive(Debug)]
pub enum JoinError {
    LeafJoined,
}

impl fmt::Display for JoinError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.description())
    }
}

impl error::Error for JoinError {
    fn description(&self) -> &str {
        match *self {
            JoinError::LeafJoined => "attempted to join leaf",
        }
    }
}

#[derive(Debug)]
pub enum SubdivideError {
    LimitExceeded,
    BranchSubdivided,
}

impl fmt::Display for SubdivideError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.description())
    }
}

impl error::Error for SubdivideError {
    fn description(&self) -> &str {
        match *self {
            SubdivideError::LimitExceeded => "minimum width limit exceeded",
            SubdivideError::BranchSubdivided => "attempted to subdivide branch",
        }
    }
}

pub trait ComputedCubeMut: ComputedCube + ops::DerefMut {
    fn join(&mut self) -> Result<&mut Self, JoinError> {
        try!(self.deref_mut().join().ok_or(JoinError::LeafJoined));
        Ok(self)
    }

    fn subdivide(&mut self) -> Result<&mut Self, SubdivideError> {
        if self.partition().width() > MIN_WIDTH {
            try!(self.deref_mut().subdivide().ok_or(SubdivideError::BranchSubdivided));
            Ok(self)
        } else {
            Err(SubdivideError::LimitExceeded)
        }
    }
}

#[derive(Clone)]
pub struct Traversal<'a> {
    cube: &'a Cube,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> Traversal<'a> {
    fn new(cube: &'a Cube, root: &'a Partition, partition: Partition) -> Self {
        Traversal {
            cube: cube,
            root: root,
            partition: partition,
        }
    }

    pub fn iter(&self) -> CubeIter {
        CubeIter::new(self.clone())
    }

    pub fn at_point(&self, point: &Point3, width: RootWidth) -> Self {
        let mut cube = self.cube;
        let mut depth = self.partition.width();

        // Clamp the inputs.
        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = exp(width.clamp(MIN_WIDTH, depth));

        while (width >> depth) == 0 {
            match *cube {
                Cube::Branch(ref branch) => {
                    depth = depth - 1;
                    cube = &branch.cubes[index_at_point(&point, depth)]
                }
                _ => break,
            }
        }

        Traversal::new(cube, self.root, Partition::at_point(&point, depth))
    }
}

impl<'a> ComputedCube for Traversal<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

impl<'a> ops::Deref for Traversal<'a> {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        self.cube
    }
}

pub struct CubeIter<'a> {
    traversals: Vec<Traversal<'a>>,
}

impl<'a> CubeIter<'a> {
    fn new(traversal: Traversal<'a>) -> Self {
        CubeIter { traversals: vec![traversal] }
    }
}

impl<'a> Iterator for CubeIter<'a> {
    type Item = Traversal<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(traversal) = self.traversals.pop() {
            match *traversal.cube {
                Cube::Branch(ref branch) => {
                    for (index, cube) in branch.iter().enumerate() {
                        self.traversals.push(Traversal::new(cube,
                                                            traversal.root,
                                                            traversal.partition()
                                                                .at_index(index)
                                                                .unwrap()));
                    }
                }
                _ => {}
            }
            Some(traversal)
        } else {
            None
        }
    }
}

pub struct TraversalMut<'a> {
    cube: &'a mut Cube,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> TraversalMut<'a> {
    fn new(cube: &'a mut Cube, root: &'a Partition, partition: Partition) -> Self {
        TraversalMut {
            cube: cube,
            root: root,
            partition: partition,
        }
    }

    pub fn at_point(&'a mut self, point: &Point3, width: RootWidth) -> Self {
        let mut cube: Option<&mut Cube> = Some(self.cube);
        let mut depth = self.partition.width();

        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = exp(width.clamp(MIN_WIDTH, depth));

        while (width >> depth) == 0 {
            let inner = cube.take().unwrap();
            match *inner {
                Cube::Branch(ref mut branch) => {
                    depth = depth - 1;
                    cube = Some(&mut branch.cubes[index_at_point(&point, depth)]);
                }
                _ => {
                    cube = Some(inner);
                    break;
                }
            }
        }

        TraversalMut::new(cube.take().unwrap(),
                          self.root,
                          Partition::at_point(&point, depth))
    }
}

impl<'a> ComputedCube for TraversalMut<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

impl<'a> ComputedCubeMut for TraversalMut<'a> {}

impl<'a> ops::Deref for TraversalMut<'a> {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        self.cube
    }
}

impl<'a> ops::DerefMut for TraversalMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cube
    }
}

pub struct Tree {
    cube: Cube,
    partition: Partition,
}

impl Tree {
    pub fn new(width: RootWidth) -> Self {
        Tree {
            cube: Cube::new(),
            partition: Partition::at_point(&Point3::origin(), width.clamp(MIN_WIDTH, MAX_WIDTH)),
        }
    }

    pub fn iter(&self) -> CubeIter {
        CubeIter::new(self.traverse())
    }

    pub fn traverse(&self) -> Traversal {
        Traversal::new(&self.cube, &self.partition, self.partition.clone())
    }

    pub fn traverse_mut(&mut self) -> TraversalMut {
        TraversalMut::new(&mut self.cube, &self.partition, self.partition.clone())
    }
}

impl ComputedCube for Tree {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        0
    }
}

impl ComputedCubeMut for Tree {}

impl ops::Deref for Tree {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        &self.cube
    }
}

impl ops::DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cube
    }
}

#[derive(Clone)]
pub enum Cube {
    Leaf(LeafNode),
    Branch(BranchNode),
}

impl Cube {
    fn new() -> Self {
        Cube::Leaf(LeafNode::new())
    }

    fn join(&mut self) -> Option<&mut Self> {
        let cube = mem::replace(self, Cube::default());
        match cube {
            Cube::Branch(branch) => {
                *self = branch.join().into();
                Some(self)
            }
            _ => {
                *self = cube;
                None
            }
        }
    }

    fn subdivide(&mut self) -> Option<&mut Self> {
        let cube = mem::replace(self, Cube::default());
        match cube {
            Cube::Leaf(leaf) => {
                *self = leaf.subdivide().into();
                Some(self)
            }
            _ => {
                *self = cube;
                None
            }
        }
    }

    pub fn is_leaf(&self) -> bool {
        match *self {
            Cube::Leaf(_) => true,
            _ => false,
        }
    }
}

impl Default for Cube {
    fn default() -> Self {
        Cube::new()
    }
}

impl From<LeafNode> for Cube {
    fn from(leaf: LeafNode) -> Self {
        Cube::Leaf(leaf)
    }
}

impl From<BranchNode> for Cube {
    fn from(branch: BranchNode) -> Self {
        Cube::Branch(branch)
    }
}

#[derive(Clone, Copy)]
pub struct LeafNode {
    pub geometry: Geometry,
    pub material: ResourceId,
}

impl LeafNode {
    fn new() -> Self {
        LeafNode {
            geometry: Geometry::full(),
            material: 0,
        }
    }

    fn subdivide(self) -> BranchNode {
        // TODO: Transform the geometry of the parent into the children.
        let cube: Cube = self.into();
        BranchNode {
            cubes: Box::new([cube.clone(),
                             cube.clone(),
                             cube.clone(),
                             cube.clone(),
                             cube.clone(),
                             cube.clone(),
                             cube.clone(),
                             cube]),
        }
    }
}

pub struct BranchNode {
    cubes: Box<[Cube; 8]>,
}

impl BranchNode {
    fn join(self) -> LeafNode {
        // TODO: Copy data from one of the original leaves.
        LeafNode::new()
    }
}

impl Clone for BranchNode {
    fn clone(&self) -> Self {
        BranchNode {
            cubes: Box::new([self.cubes[0].clone(),
                             self.cubes[1].clone(),
                             self.cubes[2].clone(),
                             self.cubes[3].clone(),
                             self.cubes[4].clone(),
                             self.cubes[5].clone(),
                             self.cubes[6].clone(),
                             self.cubes[7].clone()]),
        }
    }
}

impl ops::Deref for BranchNode {
    type Target = [Cube];

    fn deref(&self) -> &Self::Target {
        &*self.cubes
    }
}

#[derive(Copy, Clone)]
pub struct Geometry([[Edge; 4]; 3]);

impl Geometry {
    fn full() -> Self {
        Geometry([[Edge::full(); 4]; 3])
    }

    fn empty() -> Self {
        Geometry([[Edge::converged(0); 4]; 3])
    }

    pub fn as_edges_x(&self) -> &[Edge; 4] {
        &self.0[0]
    }
    pub fn as_edges_y(&self) -> &[Edge; 4] {
        &self.0[1]
    }
    pub fn as_edges_z(&self) -> &[Edge; 4] {
        &self.0[2]
    }

    pub fn as_edges_x_mut(&mut self) -> &mut [Edge; 4] {
        &mut self.0[0]
    }
    pub fn as_edges_y_mut(&mut self) -> &mut [Edge; 4] {
        &mut self.0[1]
    }
    pub fn as_edges_z_mut(&mut self) -> &mut [Edge; 4] {
        &mut self.0[2]
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Edge(u8);

const MIN_EDGE: u8 = 0;
const MAX_EDGE: u8 = 0x0F;

impl Edge {
    fn full() -> Self {
        Edge(MAX_EDGE)
    }

    fn converged(value: u8) -> Self {
        let value = nalgebra::clamp(value, MIN_EDGE, MAX_EDGE);
        Edge((value << 4) | value)
    }

    pub fn set_front(&mut self, value: u8) {
        let value = nalgebra::clamp(value, MIN_EDGE, self.back());
        self.0 = (value << 4) | self.back();
    }

    pub fn set_back(&mut self, value: u8) {
        let value = nalgebra::clamp(value, self.front(), MAX_EDGE);
        self.0 = (self.front() << 4) | value;
    }

    pub fn front(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }

    pub fn back(&self) -> u8 {
        self.0 & 0x0F
    }

    pub fn front_transform(&self) -> f32 {
        ((self.front() - MIN_EDGE) as f32) / ((MAX_EDGE - MIN_EDGE) as f32)
    }

    pub fn back_transform(&self) -> f32 {
        let range = MAX_EDGE - MIN_EDGE;
        -((range - (self.back() - MIN_EDGE)) as f32) / (range as f32)
    }
}

trait Subdivision {}

impl Subdivision for [Cube; 8] {}

trait Storage {}

impl Storage for Box<[Cube; 8]> {}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn index_at_point(point: &Point3, width: RootWidth) -> usize {
    ((((point.x >> width) & DiscreteSpace::one()) << 0) |
     (((point.y >> width) & DiscreteSpace::one()) << 1) |
     (((point.z >> width) & DiscreteSpace::one()) << 2)) as usize
}

fn vector_at_index(index: usize, width: RootWidth) -> Vector3 {
    assert!(index < 8);
    let index = index as DiscreteSpace;
    let width = exp(width);
    Vector3::new(((index >> 0) & DiscreteSpace::one()) * width,
                 ((index >> 1) & DiscreteSpace::one()) * width,
                 ((index >> 2) & DiscreteSpace::one()) * width)
}

pub fn exp(width: RootWidth) -> DiscreteSpace {
    if width > 0 {
        DiscreteSpace::one() << (width - 1)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
