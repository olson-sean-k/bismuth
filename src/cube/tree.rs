extern crate nalgebra;
extern crate num;

use nalgebra::Origin;
use std::error;
use std::error::Error;
use std::fmt;
use std::ops;

use IgnorableResult;
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

    fn to_orphan<'a>(&'a self) -> (OrphanNode<'a>, Option<&'a NodeLink>) {
        match *self {
            Node::Leaf(ref leaf) => (OrphanNode::Leaf(leaf), None),
            Node::Branch(ref nodes, ref branch) => (OrphanNode::Branch(branch), Some(nodes)),
        }
    }

    fn to_orphan_mut<'a>(&'a mut self) -> (OrphanNodeMut<'a>, Option<&'a mut NodeLink>) {
        match *self {
            Node::Leaf(ref mut leaf) => (OrphanNodeMut::Leaf(leaf), None),
            Node::Branch(ref mut nodes, ref mut branch) => {
                (OrphanNodeMut::Branch(branch), Some(nodes))
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

    pub fn to_cube(&self) -> Cube {
        Cube::new(&self.node, &self.partition, self.partition)
    }

    pub fn to_cube_mut(&mut self) -> CubeMut {
        CubeMut::new(&mut self.node, &self.partition, self.partition)
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

    pub fn to_orphan(&self) -> OrphanCube {
        let (orphan, _) = self.node.to_orphan();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn iter(&self) -> CubeIter {
        CubeIter::new(self.clone())
    }

    // TODO: Is this useful? `CubeIter` already yields fully linked `Cube`s.
    pub fn walk<F, R>(&self, f: &F)
        where F: Fn(&Cube) -> R
    {
        for node in self.iter() {
            f(&node);
        }
    }

    pub fn at_point(&self, point: &Point3, width: LogWidth) -> Cube {
        let mut node = self.node;
        let mut depth = self.partition.width();

        // Clamp the inputs.
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

    pub fn at_index(&self, index: usize) -> Option<Cube> {
        match *self.node {
            Node::Branch(ref nodes, _) => {
                self.partition
                    .at_index(index)
                    .map(|partition| Cube::new(&nodes[index], self.root, partition))
            }
            _ => None,
        }
    }
}

impl<'a> ops::Deref for Cube<'a> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        self.node
    }
}

impl<'a> From<CubeMut<'a>> for Cube<'a> {
    fn from(cube: CubeMut<'a>) -> Self {
        Cube::new(&*cube.node, cube.root, cube.partition)
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

pub struct CubeIter<'a>(Vec<Cube<'a>>);

impl<'a> CubeIter<'a> {
    fn new(cube: Cube<'a>) -> Self {
        CubeIter(vec![cube])
    }
}

impl<'a> Iterator for CubeIter<'a> {
    type Item = Cube<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = self.0.pop() {
            match *cube.node {
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

    pub fn to_orphan(&mut self) -> OrphanCubeMut {
        let (orphan, _) = self.node.to_orphan_mut();
        OrphanCubeMut::new(orphan, self.root, self.partition)
    }

    pub fn iter(&mut self) -> CubeMutIter {
        CubeMutIter::new(self)
    }

    pub fn walk<F, R>(&mut self, f: &F)
        where F: Fn(&mut CubeMut) -> R
    {
        let mut cubes = vec![CubeMut::new(self.node, self.root, self.partition)];
        while let Some(mut cube) = cubes.pop() {
            f(&mut cube);
            let (_, nodes) = cube.node.to_orphan_mut();
            if let Some(nodes) = nodes {
                for (index, node) in nodes.iter_mut().enumerate() {
                    cubes.push(CubeMut::new(node,
                                            cube.root,
                                            cube.partition.at_index(index).unwrap()));
                }
            }
        }
    }

    pub fn at_point(&mut self, point: &Point3, width: LogWidth) -> CubeMut {
        let mut node: Option<&mut Node> = Some(self.node);
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
        CubeMut::new(node.take().unwrap(),
                     self.root,
                     Partition::at_point(&point, depth))
    }

    pub fn at_index(&mut self, index: usize) -> Option<CubeMut> {
        match *self.node {
            Node::Branch(ref mut nodes, _) => {
                let root = self.root;
                self.partition
                    .at_index(index)
                    .map(move |partition| CubeMut::new(&mut nodes[index], root, partition))
            }
            _ => None,
        }
    }

    pub fn join(&mut self) -> Result<(), JoinError> {
        self.node.join()
    }

    pub fn subdivide(&mut self) -> Result<(), SubdivideError> {
        if self.partition().is_min_width() {
            Err(SubdivideError::LimitExceeded)
        }
        else {
            self.node.subdivide()
        }
    }

    pub fn subdivide_to_point(&mut self, point: &Point3, width: LogWidth) -> CubeMut {
        let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        let mut depth = width;
        let mut cube = Some(self.at_point(point, width));
        while cube.as_ref().unwrap().partition.width() > width {
            depth = depth - 1;
            let mut taken = cube.take().unwrap();
            taken.node.subdivide().ignore();
            if let Node::Branch(ref mut nodes, _) = *taken.node {
                let index = index_at_point(point, depth);
                cube = Some(CubeMut::new(&mut nodes[index],
                                         taken.root,
                                         taken.partition.at_index(index).unwrap()));
            }
        }
        cube.take().unwrap()
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

pub struct CubeMutIter<'a>(Vec<CubeMut<'a>>);

impl<'a> CubeMutIter<'a> {
    fn new(cube: &'a mut CubeMut) -> Self {
        CubeMutIter(vec![CubeMut::new(cube.node, cube.root, cube.partition)])
    }
}

impl<'a> Iterator for CubeMutIter<'a> {
    type Item = OrphanCubeMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cube) = self.0.pop() {
            let (orphan, nodes) = cube.node.to_orphan_mut();
            if let Some(nodes) = nodes {
                for (index, node) in nodes.iter_mut().enumerate() {
                    self.0.push(CubeMut::new(node,
                                             cube.root,
                                             cube.partition.at_index(index).unwrap()));
                }
            }
            Some(OrphanCubeMut::new(orphan, cube.root, cube.partition))
        }
        else {
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
    use nalgebra::Origin;
    use super::*;

    #[test]
    fn test_cube_subdivide_to_point() {
        let point = Point3::origin();
        let width = MIN_WIDTH;
        let mut root = Root::new(MAX_WIDTH);

        root.to_cube_mut().subdivide_to_point(&point, width);
        assert!(root.to_cube().at_point(&point, width).partition().width() == width);
    }
}
