extern crate nalgebra;
extern crate num;

use nalgebra::Origin;
use std::convert::{AsMut, AsRef};
use std::error;
use std::error::Error;
use std::fmt;
use std::ops;

use edit::Cursor;
use math::Clamp;
use resource::ResourceId;
use super::geometry::*;
use super::space::*;

pub type NodeLink = Box<[Node; 8]>;

pub enum Node {
    Leaf(LeafNode),
    Branch(NodeLink, BranchNode),
}

impl Node {
    fn new() -> Self {
        Node::Leaf(LeafNode::new())
    }

    pub fn is_leaf(&self) -> bool {
        match *self {
            Node::Leaf(..) => true,
            _ => false,
        }
    }

    fn to_orphan<'a>(&'a self) -> (OrphanNode<&'a LeafNode, &'a BranchNode>, Option<&'a NodeLink>) {
        match *self {
            Node::Leaf(ref leaf) => (OrphanNode::Leaf(leaf), None),
            Node::Branch(ref nodes, ref branch) => (OrphanNode::Branch(branch), Some(nodes)),
        }
    }

    fn to_orphan_mut<'a>
        (&'a mut self)
         -> (OrphanNode<&'a mut LeafNode, &'a mut BranchNode>, Option<&'a mut NodeLink>) {
        match *self {
            Node::Leaf(ref mut leaf) => (OrphanNode::Leaf(leaf), None),
            Node::Branch(ref mut nodes, ref mut branch) => {
                (OrphanNode::Branch(branch), Some(nodes))
            }
        }
    }

    fn join(&mut self) -> Result<(), JoinError> {
        if let Node::Branch(..) = *self {
            *self = Node::Leaf(LeafNode::new());
            Ok(())
        }
        else {
            Err(JoinError::AlreadyJoined)
        }
    }

    fn subdivide(&mut self) -> Result<(), SubdivideError> {
        if let Node::Leaf(..) = *self {
            *self = Node::Branch(Box::new([self.clone(),
                                           self.clone(),
                                           self.clone(),
                                           self.clone(),
                                           self.clone(),
                                           self.clone(),
                                           self.clone(),
                                           self.clone()]),
                                 BranchNode::new());
            Ok(())
        }
        else {
            Err(SubdivideError::AlreadySubdivided)
        }
    }
}

impl AsRef<Node> for Node {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Node> for Node {
    fn as_mut(&mut self) -> &mut Self {
        self
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

pub enum OrphanNode<L, B>
    where L: AsRef<LeafNode>,
          B: AsRef<BranchNode>
{
    Leaf(L),
    Branch(B),
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

impl AsRef<LeafNode> for LeafNode {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<LeafNode> for LeafNode {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone, Copy)]
pub struct BranchNode {}

impl BranchNode {
    fn new() -> Self {
        BranchNode {}
    }
}

impl AsRef<BranchNode> for BranchNode {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<BranchNode> for BranchNode {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub struct Root {
    node: Box<Node>,
    partition: Partition,
}

impl Root {
    pub fn new(width: LogWidth) -> Self {
        Root {
            node: Box::new(Node::new()),
            partition: Partition::at_point(&Point3::origin(), width),
        }
    }

    pub fn to_cube(&self) -> Cube<&Node> {
        Cube::new(&self.node, &self.partition, self.partition)
    }

    pub fn to_cube_mut(&mut self) -> Cube<&mut Node> {
        Cube::new(&mut self.node, &self.partition, self.partition)
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

#[derive(Clone)]
pub struct Cube<'a, N>
    where N: AsRef<Node>
{
    node: N,
    root: &'a Partition,
    partition: Partition,
}

impl<'a, N> Cube<'a, N>
    where N: AsRef<Node>
{
    fn new(node: N, root: &'a Partition, partition: Partition) -> Self {
        Cube {
            node: node,
            root: root,
            partition: partition,
        }
    }

    pub fn to_orphan(&self) -> OrphanCube<&LeafNode, &BranchNode> {
        let (orphan, _) = self.node.as_ref().to_orphan();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn walk<F, R>(&self, f: &F)
        where F: Fn(&Cube<&Node>) -> R
    {
        let mut cubes = vec![self.to_value()];
        while let Some(cube) = cubes.pop() {
            f(&cube);
            let (_, nodes) = cube.node.as_ref().to_orphan();
            if let Some(nodes) = nodes {
                for (index, node) in nodes.iter().enumerate() {
                    cubes.push(Cube::new(node, cube.root, cube.partition.at_index(index).unwrap()));
                }
            }
        }
    }

    pub fn at_point(&self, point: &Point3, width: LogWidth) -> Cube<&Node> {
        let mut node = self.node.as_ref();
        let mut depth = self.partition.width();

        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = width.clamp(MIN_WIDTH, depth);

        while width < depth {
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

    pub fn at_index(&self, index: usize) -> Option<Cube<&Node>> {
        match *self.node.as_ref() {
            Node::Branch(ref nodes, _) => {
                self.partition
                    .at_index(index)
                    .map(|partition| Cube::new(&nodes[index], self.root, partition))
            }
            _ => None,
        }
    }

    fn to_value(&self) -> Cube<&Node> {
        Cube::new(self.node.as_ref(), self.root, self.partition)
    }
}

impl<'a, N> Cube<'a, &'a N>
    where N: AsRef<Node>
{
    pub fn iter(&self) -> CubeIter<&N> {
        CubeIter(vec![self.clone()])
    }

    pub fn iter_cursor(&self, cursor: &'a Cursor) -> CursorIter<&N> {
        CursorIter {
            cubes: vec![self.clone()],
            cursor: cursor,
        }
    }
}

impl<'a, N> Cube<'a, N>
    where N: AsRef<Node> + AsMut<Node>
{
    pub fn to_orphan_mut(&mut self) -> OrphanCube<&mut LeafNode, &mut BranchNode> {
        let (orphan, _) = self.node.as_mut().to_orphan_mut();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn walk_mut<F, R>(&mut self, f: &F)
        where F: Fn(&mut Cube<&mut Node>) -> R
    {
        let mut cubes = vec![self.to_value_mut()];
        while let Some(mut cube) = cubes.pop() {
            f(&mut cube);
            let (_, nodes) = cube.node.as_mut().to_orphan_mut();
            if let Some(nodes) = nodes {
                for (index, node) in nodes.iter_mut().enumerate() {
                    cubes.push(Cube::new(node, cube.root, cube.partition.at_index(index).unwrap()));
                }
            }
        }
    }

    pub fn at_point_mut(&mut self, point: &Point3, width: LogWidth) -> Cube<&mut Node> {
        let mut node: Option<&mut Node> = Some(self.node.as_mut());
        let mut depth = self.partition.width();

        let point = point.clamp(0, (exp(self.root.width())) - 1);
        let width = width.clamp(MIN_WIDTH, depth);

        while width < depth {
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
        Cube::new(node.take().unwrap(),
                  self.root,
                  Partition::at_point(&point, depth))
    }

    pub fn at_index_mut(&mut self, index: usize) -> Option<Cube<&mut Node>> {
        match *self.node.as_mut() {
            Node::Branch(ref mut nodes, _) => {
                let root = self.root;
                self.partition
                    .at_index(index)
                    .map(move |partition| Cube::new(&mut nodes[index], root, partition))
            }
            _ => None,
        }
    }

    pub fn join(&mut self) -> Result<(), JoinError> {
        self.node.as_mut().join()
    }

    pub fn subdivide(&mut self) -> Result<(), SubdivideError> {
        if self.partition().is_min_width() {
            Err(SubdivideError::LimitExceeded)
        }
        else {
            self.node.as_mut().subdivide()
        }
    }

    pub fn subdivide_to_point(&mut self, point: &Point3, width: LogWidth) -> Cube<&mut Node> {
        let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        let cube = self.at_point_mut(point, width);
        let mut depth = cube.partition.width();
        let mut node: Option<&mut Node> = Some(cube.node.as_mut());
        while depth > width {
            depth = depth - 1;
            let mut taken = node.take().unwrap();
            let _ = taken.subdivide();
            if let Node::Branch(ref mut nodes, _) = *taken {
                node = Some(&mut nodes[index_at_point(point, depth)]);
            }
        }
        Cube::new(node.take().unwrap(),
                  self.root,
                  Partition::at_point(point, depth))
    }

    pub fn subdivide_to_cursor(&mut self, cursor: &Cursor) -> Vec<Cube<&mut Node>> {
        let mut cubes = vec![];
        let mut traversal = vec![self.to_value_mut()];
        while let Some(cube) = traversal.pop() {
            if cube.aabb().intersects(&cursor.aabb()) {
                if cube.partition.width() == cursor.width() {
                    cubes.push(cube);
                }
                else if cube.partition.width() > cursor.width() {
                    let _ = cube.node.as_mut().subdivide();
                    let (_, nodes) = cube.node.as_mut().to_orphan_mut();
                    if let Some(nodes) = nodes {
                        for (index, node) in nodes.iter_mut().enumerate() {
                            traversal.push(Cube::new(node,
                                                     cube.root,
                                                     cube.partition.at_index(index).unwrap()));
                        }
                    }
                }
            }
        }
        cubes
    }

    fn to_value_mut(&mut self) -> Cube<&mut Node> {
        Cube::new(self.node.as_mut(), self.root, self.partition)
    }
}

impl<'a, N> Cube<'a, &'a mut N>
    where N: AsRef<Node> + AsMut<Node>
{
    pub fn iter_mut(&mut self) -> CubeIter<&mut N> {
        CubeIter(vec![Cube::new(self.node, self.root, self.partition)])
    }

    pub fn iter_cursor_mut(&mut self, cursor: &'a Cursor) -> CursorIter<&mut N> {
        CursorIter {
            cubes: vec![Cube::new(&mut *self.node, self.root, self.partition)],
            cursor: cursor,
        }
    }
}

impl<'a, N> ops::Deref for Cube<'a, N>
    where N: AsRef<Node>
{
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        self.node.as_ref()
    }
}

impl<'a, N> ops::DerefMut for Cube<'a, N>
    where N: AsRef<Node> + AsMut<Node>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node.as_mut()
    }
}

impl<'a, N> Spatial for Cube<'a, N>
    where N: AsRef<Node>
{
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

pub struct CubeIter<'a, N>(Vec<Cube<'a, N>>) where N: AsRef<Node>;

impl<'a> Iterator for CubeIter<'a, &'a Node> {
    type Item = Cube<'a, &'a Node>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = self.0.pop() {
            match *cube.node.as_ref() {
                Node::Branch(ref nodes, _) => {
                    for (index, node) in nodes.iter().enumerate() {
                        self.0.push(Cube::new(node,
                                              cube.root,
                                              cube.partition().at_index(index).unwrap()));
                    }
                }
                _ => {}
            }
            Some(cube)
        }
        else {
            None
        }
    }
}

impl<'a> Iterator for CubeIter<'a, &'a mut Node> {
    type Item = OrphanCube<'a, &'a mut LeafNode, &'a mut BranchNode>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = self.0.pop() {
            let (orphan, nodes) = cube.node.as_mut().to_orphan_mut();
            if let Some(nodes) = nodes {
                for (index, node) in nodes.iter_mut().enumerate() {
                    self.0.push(Cube::new(node,
                                          cube.root,
                                          cube.partition.at_index(index).unwrap()));
                }
            }
            Some(OrphanCube::new(orphan, cube.root, cube.partition))
        }
        else {
            None
        }
    }
}

pub struct CursorIter<'a, N>
    where N: AsRef<Node>
{
    cubes: Vec<Cube<'a, N>>,
    cursor: &'a Cursor,
}

impl<'a> Iterator for CursorIter<'a, &'a Node> {
    type Item = Cube<'a, &'a Node>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cube) = self.cubes.pop() {
            if cube.aabb().intersects(&self.cursor.aabb()) {
                if cube.partition.width() == self.cursor.width() || cube.node.as_ref().is_leaf() {
                    return Some(cube);
                }
                else if cube.partition.width() > self.cursor.width() {
                    let (_, nodes) = cube.node.as_ref().to_orphan();
                    if let Some(nodes) = nodes {
                        for (index, node) in nodes.iter().enumerate() {
                            self.cubes.push(Cube::new(node,
                                                      cube.root,
                                                      cube.partition.at_index(index).unwrap()));
                        }
                    }
                }
            }
        }
        None
    }
}

impl<'a> Iterator for CursorIter<'a, &'a mut Node> {
    type Item = Cube<'a, &'a mut Node>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(cube) = self.cubes.pop() {
            if cube.aabb().intersects(&self.cursor.aabb()) {
                if cube.partition.width() == self.cursor.width() || cube.node.as_ref().is_leaf() {
                    return Some(cube);
                }
                else if cube.partition.width() > self.cursor.width() {
                    let (_, nodes) = cube.node.as_mut().to_orphan_mut();
                    if let Some(nodes) = nodes {
                        for (index, node) in nodes.iter_mut().enumerate() {
                            self.cubes.push(Cube::new(node,
                                                      cube.root,
                                                      cube.partition.at_index(index).unwrap()));
                        }
                    }
                }
            }
        }
        None
    }
}

pub struct OrphanCube<'a, L, B>
    where L: AsRef<LeafNode>,
          B: AsRef<BranchNode>
{
    node: OrphanNode<L, B>,
    root: &'a Partition,
    partition: Partition,
}

impl<'a, L, B> OrphanCube<'a, L, B>
    where L: AsRef<LeafNode>,
          B: AsRef<BranchNode>
{
    fn new(node: OrphanNode<L, B>, root: &'a Partition, partition: Partition) -> Self {
        OrphanCube {
            node: node,
            root: root,
            partition: partition,
        }
    }
}

impl<'a, L, B> ops::Deref for OrphanCube<'a, L, B>
    where L: AsRef<LeafNode>,
          B: AsRef<BranchNode>
{
    type Target = OrphanNode<L, B>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, L, B> ops::DerefMut for OrphanCube<'a, L, B>
    where L: AsRef<LeafNode> + AsMut<LeafNode>,
          B: AsRef<BranchNode> + AsMut<BranchNode>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<'a, L, B> Spatial for OrphanCube<'a, L, B>
    where L: AsRef<LeafNode>,
          B: AsRef<BranchNode>
{
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width() - self.partition.width()
    }
}

#[derive(Debug)]
pub enum JoinError {
    AlreadyJoined,
}

impl fmt::Display for JoinError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.description())
    }
}

impl error::Error for JoinError {
    fn description(&self) -> &str {
        match *self {
            JoinError::AlreadyJoined => "attempted to join leaf",
        }
    }
}

#[derive(Debug)]
pub enum SubdivideError {
    LimitExceeded,
    AlreadySubdivided,
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
            SubdivideError::AlreadySubdivided => "attempted to subdivide branch",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
