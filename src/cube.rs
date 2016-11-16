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
pub struct Cube<'a> {
    node: &'a Node,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> Cube<'a> {
    fn new(node: &'a Node, root: &'a Partition, partition: Partition) -> Self {
        Cube {
            node: node,
            root: root,
            partition: partition,
        }
    }

    pub fn iter(&self) -> CubeIter {
        CubeIter::new(self.clone())
    }

    pub fn walk<F, R>(&'a self, f: &F)
        where F: Fn(Cube<'a>) -> R
    {
        for node in self.iter() {
            f(node);
        }
    }

    pub fn at_point(&self, point: &Point3, width: RootWidth) -> Self {
        let mut node = self.node;
        let mut depth = self.partition.width();

        // Clamp the inputs.
        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = exp(width.clamp(MIN_WIDTH, depth));

        while (width >> depth) == 0 {
            match *node {
                Node::Branch(ref nodes, _) => {
                    depth = depth - 1;
                    node = &nodes[index_at_point(&point, depth)]
                }
                _ => break,
            }
        }

        Cube::new(node, self.root, Partition::at_point(&point, depth))
    }

    pub fn at_index(&self, index: usize) -> Option<Self> {
        unimplemented!()
    }
}

impl<'a> ops::Deref for Cube<'a> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        self.node
    }
}

impl<'a> Spatial for Cube<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct CubeIter<'a> {
    cubes: Vec<Cube<'a>>,
}

impl<'a> CubeIter<'a> {
    fn new(cube: Cube<'a>) -> Self {
        CubeIter { cubes: vec![cube] }
    }
}

impl<'a> Iterator for CubeIter<'a> {
    type Item = Cube<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = self.cubes.pop() {
            match *cube.node {
                Node::Branch(ref nodes, _) => {
                    for (index, node) in nodes.iter().enumerate() {
                        self.cubes.push(Cube::new(node,
                                                  cube.root,
                                                  cube.partition().at_index(index).unwrap()));
                    }
                }
                _ => {}
            }
            Some(cube)
        } else {
            None
        }
    }
}

pub struct CubeMut<'a> {
    node: &'a mut Node,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> CubeMut<'a> {
    fn new(node: &'a mut Node, root: &'a Partition, partition: Partition) -> Self {
        CubeMut {
            node: node,
            root: root,
            partition: partition,
        }
    }

    pub fn iter(&'a mut self) -> CubeMutIter {
        CubeMutIter::new(self)
    }

    pub fn walk<F, R>(&'a mut self, f: &F)
        where F: Fn(&mut CubeMut) -> R
    {
        f(self);
        if let Node::Branch(ref mut nodes, _) = *self.node {
            for (index, node) in nodes.iter_mut().enumerate() {
                let mut cube =
                    CubeMut::new(node, self.root, self.partition.at_index(index).unwrap());
                cube.walk(f);
            }
        }
    }

    pub fn at_point(&'a mut self, point: &Point3, width: RootWidth) -> Self {
        let mut node: Option<&mut Node> = Some(self.node);
        let mut depth = self.partition.width();

        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = exp(width.clamp(MIN_WIDTH, depth));

        while (width >> depth) == 0 {
            let taken = node.take().unwrap();
            match *taken {
                Node::Branch(ref mut nodes, _) => {
                    depth = depth - 1;
                    node = Some(&mut nodes[index_at_point(&point, depth)]);
                }
                _ => {
                    node = Some(taken);
                    break;
                }
            }
        }

        CubeMut::new(node.take().unwrap(),
                     self.root,
                     Partition::at_point(&point, depth))
    }

    pub fn at_index(&mut self, index: usize) -> Option<Self> {
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

impl<'a> ops::Deref for CubeMut<'a> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &*self.node
    }
}

impl<'a> ops::DerefMut for CubeMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node
    }
}

impl<'a> Spatial for CubeMut<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct CubeMutIter<'a> {
    cubes: Vec<CubeMut<'a>>,
}

impl<'a> CubeMutIter<'a> {
    fn new(cube: &'a mut CubeMut<'a>) -> Self {
        CubeMutIter { cubes: vec![CubeMut::new(cube.node, cube.root, cube.partition.clone())] }
    }
}

impl<'a> Iterator for CubeMutIter<'a> {
    type Item = OrphanCubeMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = self.cubes.pop() {
            let (orphan, nodes) = cube.node.to_orphan_mut();
            if let Some(nodes) = nodes {
                for (index, node) in nodes.iter_mut().enumerate() {
                    self.cubes.push(CubeMut::new(node,
                                                 cube.root,
                                                 cube.partition.at_index(index).unwrap()));
                }
            }
            Some(OrphanCubeMut::new(orphan, cube.root, cube.partition.clone()))
        } else {
            None
        }
    }
}

pub struct OrphanCube<'a> {
    node: OrphanNode<'a>,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> OrphanCube<'a> {
    fn new(node: OrphanNode<'a>, root: &'a Partition, partition: Partition) -> Self {
        OrphanCube {
            node: node,
            root: root,
            partition: partition,
        }
    }
}

impl<'a> ops::Deref for OrphanCube<'a> {
    type Target = OrphanNode<'a>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a> Spatial for OrphanCube<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct OrphanCubeMut<'a> {
    node: OrphanNodeMut<'a>,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> OrphanCubeMut<'a> {
    fn new(node: OrphanNodeMut<'a>, root: &'a Partition, partition: Partition) -> Self {
        OrphanCubeMut {
            node: node,
            root: root,
            partition: partition,
        }
    }
}

impl<'a> ops::Deref for OrphanCubeMut<'a> {
    type Target = OrphanNodeMut<'a>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a> ops::DerefMut for OrphanCubeMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<'a> Spatial for OrphanCubeMut<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct Root {
    node: Node,
    partition: Partition,
}

impl Root {
    pub fn new(width: RootWidth) -> Self {
        Root {
            node: Node::new(),
            partition: Partition::at_point(&Point3::origin(), width.clamp(MIN_WIDTH, MAX_WIDTH)),
        }
    }

    pub fn to_cube(&self) -> Cube {
        Cube::new(&self.node, &self.partition, self.partition.clone())
    }

    pub fn to_cube_mut(&mut self) -> CubeMut {
        CubeMut::new(&mut self.node, &self.partition, self.partition.clone())
    }
}

impl Spatial for Root {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        0
    }
}

pub type NodeLink = Box<[Node; 8]>;

pub enum Node {
    Leaf(LeafNode),
    Branch(NodeLink, BranchNode),
}

impl Node {
    fn new() -> Self {
        Node::Leaf(LeafNode::new())
    }

    pub fn to_orphan<'a>(&'a self) -> (OrphanNode<'a>, Option<&'a NodeLink>) {
        match *self {
            Node::Leaf(ref leaf) => (OrphanNode::Leaf(leaf), None),
            Node::Branch(ref nodes, ref branch) => (OrphanNode::Branch(branch), Some(nodes)),
        }
    }

    pub fn to_orphan_mut<'a>(&'a mut self) -> (OrphanNodeMut<'a>, Option<&'a mut NodeLink>) {
        match *self {
            Node::Leaf(ref mut leaf) => (OrphanNodeMut::Leaf(leaf), None),
            Node::Branch(ref mut nodes, ref mut branch) => {
                (OrphanNodeMut::Branch(branch), Some(nodes))
            },
        }
    }

    pub fn is_leaf(&self) -> bool {
        match *self {
            Node::Leaf(_) => true,
            _ => false,
        }
    }

    fn join(&mut self) -> Option<&mut Self> {
        let node = mem::replace(self, Node::default());
        match node {
            Node::Branch(_, _) => {
                *self = Node::Leaf(LeafNode::new());
                Some(self)
            }
            _ => {
                *self = node;
                None
            }
        }
    }

    fn subdivide(&mut self) -> Option<&mut Self> {
        let node = mem::replace(self, Node::default());
        match node {
            Node::Leaf(node) => {
                let node = Node::new();
                *self = Node::Branch(Box::new([node.clone(),
                                               node.clone(),
                                               node.clone(),
                                               node.clone(),
                                               node.clone(),
                                               node.clone(),
                                               node.clone(),
                                               node]),
                                     BranchNode::new());
                Some(self)
            }
            _ => {
                *self = node;
                None
            }
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        match *self {
            Node::Leaf(leaf) => Node::Leaf(leaf.clone()),
            Node::Branch(ref nodes, branch) => {
                Node::Branch(Box::new([nodes[0].clone(),
                                       nodes[1].clone(),
                                       nodes[2].clone(),
                                       nodes[3].clone(),
                                       nodes[4].clone(),
                                       nodes[5].clone(),
                                       nodes[6].clone(),
                                       nodes[7].clone()]),
                             branch.clone())
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::new()
    }
}

pub enum OrphanNode<'a> {
    Leaf(&'a LeafNode),
    Branch(&'a BranchNode),
}

pub enum OrphanNodeMut<'a> {
    Leaf(&'a mut LeafNode),
    Branch(&'a mut BranchNode),
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
}

#[derive(Clone, Copy)]
pub struct BranchNode {}

impl BranchNode {
    fn new() -> Self {
        BranchNode {}
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
