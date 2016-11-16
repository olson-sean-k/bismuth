//! This module provides an oct-tree of cubes. Each leaf cube describes a
//! spatial partition, its properties, and a deformable geometry.

extern crate nalgebra;
extern crate num;

use nalgebra::Origin;
use num::{One, Zero}; // TODO: `use ::std::num::{One, Zero};`.
use std::error;
use std::error::Error;
use std::fmt;
use std::mem;
use std::ops;
use std::ops::DerefMut;

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

pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

pub enum Direction {
    Positive,
    Negative,
}

pub enum Orientation {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

impl Orientation {
    pub fn axis(&self) -> Axis {
        match *self {
            Orientation::Left | Orientation::Right => Axis::X,
            Orientation::Top | Orientation::Bottom => Axis::Y,
            Orientation::Front | Orientation::Back => Axis::Z,
        }
    }

    pub fn direction(&self) -> Direction {
        match *self {
            Orientation::Left | Orientation::Top | Orientation::Front => Direction::Positive,
            Orientation::Right | Orientation::Bottom | Orientation::Back => Direction::Negative,
        }
    }
}

#[derive(Clone)]
pub struct Partition {
    origin: Point3,
    width: RootWidth,
}

impl Partition {
    pub fn at_point(point: &Point3, width: RootWidth) -> Self {
        Partition {
            origin: point.mask(!DiscreteSpace::zero() << width),
            width: width,
        }
    }

    pub fn at_index(&self, index: usize) -> Option<Self> {
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
        self.origin + Vector3::new(m, m, m)
    }
}

pub trait Spatial {
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

#[derive(Clone)]
pub struct Tree<'a> {
    cube: &'a Cube,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> Tree<'a> {
    fn new(cube: &'a Cube, root: &'a Partition, partition: Partition) -> Self {
        Tree {
            cube: cube,
            root: root,
            partition: partition,
        }
    }

    pub fn iter(&self) -> TreeIter {
        TreeIter::new(self.clone())
    }

    pub fn walk<F, R>(&'a self, f: &F)
        where F: Fn(Tree<'a>) -> R
    {
        for cube in self.iter() {
            f(cube);
        }
    }

    pub fn at_point(&self, point: &Point3, width: RootWidth) -> Self {
        let mut cube = self.cube;
        let mut depth = self.partition.width();

        // Clamp the inputs.
        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = exp(width.clamp(MIN_WIDTH, depth));

        while (width >> depth) == 0 {
            match *cube {
                Cube::Branch(ref cubes, _) => {
                    depth = depth - 1;
                    cube = &cubes[index_at_point(&point, depth)]
                },
                _ => break,
            }
        }

        Tree::new(cube, self.root, Partition::at_point(&point, depth))
    }

    pub fn at_index(&self, index: usize) -> Self {
        unimplemented!()
    }
}

impl<'a> ops::Deref for Tree<'a> {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        self.cube
    }
}

impl<'a> Spatial for Tree<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct TreeIter<'a> {
    trees: Vec<Tree<'a>>,
}

impl<'a> TreeIter<'a> {
    fn new(tree: Tree<'a>) -> Self {
        TreeIter { trees: vec![tree] }
    }
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = Tree<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tree) = self.trees.pop() {
            match *tree.cube {
                Cube::Branch(ref cubes, _) => {
                    for (index, cube) in cubes.iter().enumerate() {
                        self.trees.push(Tree::new(cube,
                                                  tree.root,
                                                  tree.partition().at_index(index).unwrap()));
                    }
                }
                _ => {}
            }
            Some(tree)
        } else {
            None
        }
    }
}

pub struct TreeMut<'a> {
    cube: &'a mut Cube,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> TreeMut<'a> {
    fn new(cube: &'a mut Cube, root: &'a Partition, partition: Partition) -> Self {
        TreeMut {
            cube: cube,
            root: root,
            partition: partition,
        }
    }

    pub fn iter(&'a mut self) -> TreeMutIter {
        TreeMutIter::new(self)
    }

    pub fn walk<F, R>(&'a mut self, f: &F)
        where F: Fn(&mut TreeMut) -> R
    {
        f(self);
        if let Cube::Branch(ref mut cubes, _) = *self.cube {
            for (index, cube) in cubes.iter_mut().enumerate() {
                let mut tree = TreeMut::new(cube,
                                            self.root,
                                            self.partition.at_index(index).unwrap());
                tree.walk(f);
            }
        }
    }

    pub fn at_point(&'a mut self, point: &Point3, width: RootWidth) -> Self {
        let mut cube: Option<&mut Cube> = Some(self.cube);
        let mut depth = self.partition.width();

        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = exp(width.clamp(MIN_WIDTH, depth));

        while (width >> depth) == 0 {
            let taken = cube.take().unwrap();
            match *taken {
                Cube::Branch(ref mut cubes, _) => {
                    depth = depth - 1;
                    cube = Some(&mut cubes[index_at_point(&point, depth)]);
                }
                _ => {
                    cube = Some(taken);
                    break;
                }
            }
        }

        TreeMut::new(cube.take().unwrap(), self.root, Partition::at_point(&point, depth))
    }

    pub fn at_index(&mut self, index: usize) -> Self {
        unimplemented!()
    }

    pub fn join(&mut self) -> Result<&mut Self, JoinError> {
        self.deref_mut().join().ok_or(JoinError::LeafJoined)?;
        Ok(self)
    }

    pub fn subdivide(&mut self) -> Result<&mut Self, SubdivideError> {
        if self.partition().width() > MIN_WIDTH {
            self.deref_mut().subdivide().ok_or(SubdivideError::BranchSubdivided)?;
            Ok(self)
        } else {
            Err(SubdivideError::LimitExceeded)
        }
    }
}

impl<'a> ops::Deref for TreeMut<'a> {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        &*self.cube
    }
}

impl<'a> ops::DerefMut for TreeMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.cube
    }
}

impl<'a> Spatial for TreeMut<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct TreeMutIter<'a> {
    trees: Vec<TreeMut<'a>>,
}

impl<'a> TreeMutIter<'a> {
    fn new(tree: &'a mut TreeMut<'a>) -> Self {
        TreeMutIter {
            trees: vec![TreeMut::new(tree.cube, tree.root, tree.partition.clone())],
        }
    }
}

impl<'a> Iterator for TreeMutIter<'a> {
    type Item = OrphanMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tree) = self.trees.pop() {
            let (orphan, cubes) = tree.cube.to_orphan_mut();
            if let Some(cubes) = cubes {
                for (index, cube) in cubes.iter_mut().enumerate() {
                    self.trees.push(TreeMut::new(cube,
                                                 tree.root,
                                                 tree.partition.at_index(index).unwrap()));
                }
            }
            Some(OrphanMut::new(orphan, tree.root, tree.partition.clone()))
        } else {
            None
        }
    }
}

pub struct Orphan<'a> {
    cube: OrphanCube<'a>,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> Orphan<'a> {
    fn new(cube: OrphanCube<'a>, root: &'a Partition, partition: Partition) -> Self {
        Orphan {
            cube: cube,
            root: root,
            partition: partition,
        }
    }
}

impl<'a> ops::Deref for Orphan<'a> {
    type Target = OrphanCube<'a>;

    fn deref(&self) -> &Self::Target {
        &self.cube
    }
}

impl<'a> Spatial for Orphan<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct OrphanMut<'a> {
    cube: OrphanCubeMut<'a>,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> OrphanMut<'a> {
    fn new(cube: OrphanCubeMut<'a>, root: &'a Partition, partition: Partition) -> Self {
        OrphanMut {
            cube: cube,
            root: root,
            partition: partition,
        }
    }
}

impl<'a> ops::Deref for OrphanMut<'a> {
    type Target = OrphanCubeMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.cube
    }
}

impl<'a> ops::DerefMut for OrphanMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cube
    }
}

impl<'a> Spatial for OrphanMut<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct Root {
    cube: Cube,
    partition: Partition,
}

impl Root {
    pub fn new(width: RootWidth) -> Self {
        Root {
            cube: Cube::new(),
            partition: Partition::at_point(&Point3::origin(), width.clamp(MIN_WIDTH, MAX_WIDTH)),
        }
    }

    pub fn tree(&self) -> Tree {
        Tree::new(&self.cube, &self.partition, self.partition.clone())
    }

    pub fn tree_mut(&mut self) -> TreeMut {
        TreeMut::new(&mut self.cube, &self.partition, self.partition.clone())
    }
}

impl Spatial for Root {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        0u8
    }
}

pub type CubeLink = Box<[Cube; 8]>;

pub enum Cube {
    Leaf(LeafCube),
    Branch(CubeLink, BranchCube),
}

impl Cube {
    fn new() -> Self {
        Cube::Leaf(LeafCube::new())
    }

    pub fn to_orphan<'a>(&'a self) -> (OrphanCube<'a>, Option<&'a CubeLink>) {
        match *self {
            Cube::Leaf(ref leaf) => (OrphanCube::Leaf(leaf), None),
            Cube::Branch(ref cubes, ref branch) => (OrphanCube::Branch(branch), Some(cubes)),
        }
    }

    pub fn to_orphan_mut<'a>(&'a mut self) -> (OrphanCubeMut<'a>, Option<&'a mut CubeLink>) {
        match *self {
            Cube::Leaf(ref mut leaf) => (OrphanCubeMut::Leaf(leaf), None),
            Cube::Branch(ref mut cubes, ref mut branch) => (OrphanCubeMut::Branch(branch), Some(cubes)),
        }
    }

    pub fn is_leaf(&self) -> bool {
        match *self {
            Cube::Leaf(_) => true,
            _ => false,
        }
    }

    fn join(&mut self) -> Option<&mut Self> {
        let cube = mem::replace(self, Cube::default());
        match cube {
            Cube::Branch(_, _) => {
                *self = Cube::Leaf(LeafCube::new());
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
            Cube::Leaf(cube) => {
                let cube = Cube::new();
                *self = Cube::Branch(Box::new([cube.clone(),
                                               cube.clone(),
                                               cube.clone(),
                                               cube.clone(),
                                               cube.clone(),
                                               cube.clone(),
                                               cube.clone(),
                                               cube]),
                                     BranchCube::new());
                Some(self)
            }
            _ => {
                *self = cube;
                None
            }
        }
    }
}

impl Clone for Cube {
    fn clone(&self) -> Self {
        match *self {
            Cube::Leaf(leaf) => {
                Cube::Leaf(leaf.clone())
            },
            Cube::Branch(ref cubes, branch) => {
                Cube::Branch(Box::new([cubes[0].clone(),
                                       cubes[1].clone(),
                                       cubes[2].clone(),
                                       cubes[3].clone(),
                                       cubes[4].clone(),
                                       cubes[5].clone(),
                                       cubes[6].clone(),
                                       cubes[7].clone()]),
                             branch.clone())
            },
        }
    }
}

impl Default for Cube {
    fn default() -> Self {
        Cube::new()
    }
}

pub enum OrphanCube<'a> {
    Leaf(&'a LeafCube),
    Branch(&'a BranchCube),
}

pub enum OrphanCubeMut<'a> {
    Leaf(&'a mut LeafCube),
    Branch(&'a mut BranchCube),
}

#[derive(Clone, Copy)]
pub struct LeafCube {
    pub geometry: Geometry,
    pub material: ResourceId,
}

impl LeafCube {
    fn new() -> Self {
        LeafCube {
            geometry: Geometry::full(),
            material: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BranchCube {}

impl BranchCube {
    fn new() -> Self {
        BranchCube {}
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

    pub fn axis_edges(&self, axis: Axis) -> &[Edge; 4] {
        &self.0[axis as usize]
    }

    pub fn axis_edges_mut(&mut self, axis: Axis) -> &mut [Edge; 4] {
        &mut self.0[axis as usize]
    }
}

#[derive(Copy, Clone)]
pub struct Edge(u8);

pub const MIN_EDGE: u8 = 0;
pub const MAX_EDGE: u8 = 0x0F;

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
}

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
