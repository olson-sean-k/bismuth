//! This module provides an oct-tree of cubes. Each leaf cube describes a
//! spatial partition, its properties, and a deformable geometry.

extern crate nalgebra;

use ::nalgebra::Origin;
use ::std::error;
use ::std::error::Error;
use ::std::fmt;
use ::std::mem;
use ::std::ops;
use ::std::ops::Deref;

use resource::ResourceId;

pub const MAX_WIDTH: RootWidth = 32;
pub const MIN_WIDTH: RootWidth = 4;

pub type Point = nalgebra::Point3<u32>;
pub type Vector = nalgebra::Vector3<u32>;
pub type RootWidth = u8; // TODO: https://github.com/rust-lang/rfcs/issues/671

pub trait Clamp<T>
    where T: PartialOrd
{
    fn clamp(&self, min: T, max: T) -> Self;
}

impl Clamp<RootWidth> for RootWidth {
    fn clamp(&self, min: RootWidth, max: RootWidth) -> Self {
        nalgebra::clamp(*self, min, max)
    }
}

impl<T> Clamp<T> for nalgebra::Point3<T>
    where T: Copy,
          T: PartialOrd
{
    fn clamp(&self, min: T, max: T) -> Self {
        nalgebra::Point3::new(nalgebra::clamp(self.x, min, max),
                              nalgebra::clamp(self.y, min, max),
                              nalgebra::clamp(self.z, min, max))
    }
}

pub trait Mask<T>
    where T: ops::BitAnd<Output = T>
{
    fn mask(&self, value: T) -> Self;
}

impl<T> Mask<T> for nalgebra::Point3<T>
    where T: Copy,
          T: ops::BitAnd<Output = T>
{
    fn mask(&self, value: T) -> Self {
        nalgebra::Point3::new(self.x & value, self.y & value, self.z & value)
    }
}

#[derive(Clone)]
pub struct Partition {
    origin: Point,
    width: RootWidth,
}

impl Partition {
    fn at_point(point: &Point, width: RootWidth) -> Self {
        Partition {
            origin: point.mask(!0u32 << width),
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
        }
        else {
            None
        }
    }

    pub fn origin(&self) -> &Point {
        &self.origin
    }

    pub fn width(&self) -> RootWidth {
        self.width
    }
}

pub trait Traversal: ops::Deref<Target = Cube> {
    fn partition(&self) -> &Partition;

    fn depth(&self) -> u8;
}

pub trait TraversalMut: Traversal + ops::DerefMut {}

#[derive(Clone)]
pub struct Cursor<'a> {
    cube: &'a Cube,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> Cursor<'a> {
    fn new(cube: &'a Cube, root: &'a Partition, partition: Partition) -> Self {
        Cursor {
            cube: cube,
            root: root,
            partition: partition,
        }
    }

    pub fn iter(&self) -> CursorIter {
        CursorIter::new(self.clone())
    }

    pub fn resolve(&self, point: &Point, width: RootWidth) -> Self {
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

        Cursor::new(cube, self.root, Partition::at_point(&point, depth))
    }
}

impl<'a> Traversal for Cursor<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

impl<'a> ops::Deref for Cursor<'a> {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        self.cube
    }
}

pub struct CursorIter<'a> {
    cursors: Vec<Cursor<'a>>,
}

impl<'a> CursorIter<'a> {
    fn new(cursor: Cursor<'a>) -> Self {
        CursorIter {
            cursors: vec![cursor],
        }
    }
}

impl<'a> Iterator for CursorIter<'a> {
    type Item = Cursor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cursor) = self.cursors.pop() {
            match *cursor.cube {
                Cube::Branch(ref branch) => {
                    for (index, cube) in branch.iter().enumerate() {
                        self.cursors.push(Cursor::new(
                            cube,
                            cursor.root,
                            cursor.partition().at_index(index).unwrap()));
                    }
                },
                _ => {},
            }
            Some(cursor)
        }
        else {
            None
        }
    }
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

pub struct CursorMut<'a> {
    cube: &'a mut Cube,
    root: &'a Partition,
    partition: Partition,
}

impl<'a> CursorMut<'a> {
    fn new(cube: &'a mut Cube, root: &'a Partition, partition: Partition) -> Self {
        CursorMut {
            cube: cube,
            root: root,
            partition: partition,
        }
    }

    pub fn resolve(&'a mut self, point: &Point, width: RootWidth) -> Self {
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

        CursorMut::new(cube.take().unwrap(),
                       self.root,
                       Partition::at_point(&point, depth))
    }

    pub fn join(&mut self) -> Result<&mut Self, JoinError> {
        try!(self.cube.join().ok_or(JoinError::LeafJoined));
        Ok(self)
    }

    pub fn subdivide(&mut self) -> Result<&mut Self, SubdivideError> {
        if self.partition.width() > MIN_WIDTH {
            try!(self.cube.subdivide().ok_or(SubdivideError::BranchSubdivided));
            Ok(self)
        } else {
            Err(SubdivideError::LimitExceeded)
        }
    }
}

impl<'a> Traversal for CursorMut<'a> {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

impl<'a> TraversalMut for CursorMut<'a> {}

impl<'a> ops::Deref for CursorMut<'a> {
    type Target = Cube;

    fn deref(&self) -> &Self::Target {
        self.cube
    }
}

impl<'a> ops::DerefMut for CursorMut<'a> {
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
            partition: Partition::at_point(&Point::origin(), width),
        }
    }

    pub fn cursor(&self) -> Cursor {
        Cursor::new(&self.cube, &self.partition, self.partition.clone())
    }

    pub fn cursor_mut(&mut self) -> CursorMut {
        CursorMut::new(&mut self.cube, &self.partition, self.partition.clone())
    }
}

impl Traversal for Tree {
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        0
    }
}

impl TraversalMut for Tree {}

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
    pub geometry: [u32; 3],
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

trait Geometry {
    fn full() -> Self;
    fn empty() -> Self;
}

impl Geometry for [u32; 3] {
    fn full() -> Self {
        [0x0F0F0F0F; 3]
    }

    fn empty() -> Self {
        [0x00000000; 3]
    }
}

trait Subdivision {}

impl Subdivision for [Cube; 8] {}

trait Storage {}

impl Storage for Box<[Cube; 8]> {}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn index_at_point(point: &Point, width: RootWidth) -> usize {
    ((((point.x >> width) & 1u32) << 0) |
     (((point.y >> width) & 1u32) << 1) |
     (((point.z >> width) & 1u32) << 2)) as usize
}

fn vector_at_index(index: usize, width: RootWidth) -> Vector {
    assert!(index < 8);
    let index = index as u32;
    let width = exp(width);
    Vector::new(
        ((index >> 0) & 1u32) * width,
        ((index >> 1) & 1u32) * width,
        ((index >> 2) & 1u32) * width)
}

pub fn exp(width: RootWidth) -> u32 {
    if width > 0 {
        1u32 << (width - 1)
    }
    else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
