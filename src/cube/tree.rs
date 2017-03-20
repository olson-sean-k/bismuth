use num::Bounded;
use std::convert::{AsMut, AsRef};
use std::error;
use std::error::Error;
use std::fmt;
use std::ops;

use math::{Clamp, FRay3, FScalar, UPoint3};
use resource::ResourceId;
use super::edit::Cursor;
use super::geometry::Geometry;
use super::space::{self, Intersects, LogWidth, Partition, RayCast, RayIntersection, Spatial};
use super::traverse::{Trace, Traversal};

type NodeLink = Box<[Node; 8]>;

pub enum Node {
    Leaf(LeafNode),
    Branch(BranchNode),
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

    pub fn as_leaf(&self) -> Option<&LeafNode> {
        match *self {
            Node::Leaf(ref leaf) => Some(leaf),
            _ => None,
        }
    }

    pub fn as_leaf_mut(&mut self) -> Option<&mut LeafNode> {
        match *self {
            Node::Leaf(ref mut leaf) => Some(leaf),
            _ => None,
        }
    }

    pub fn as_branch(&self) -> Option<&BranchNode> {
        match *self {
            Node::Branch(ref branch) => Some(branch),
            _ => None,
        }
    }

    pub fn as_branch_mut(&mut self) -> Option<&mut BranchNode> {
        match *self {
            Node::Branch(ref mut branch) => Some(branch),
            _ => None,
        }
    }

    fn hint(&self) -> &Hint {
        match *self {
            Node::Leaf(ref leaf) => &leaf.hint,
            Node::Branch(ref branch) => &branch.hint,
        }
    }

    #[allow(dead_code)]
    fn hint_mut(&mut self) -> &mut Hint {
        match *self {
            Node::Leaf(ref mut leaf) => &mut leaf.hint,
            Node::Branch(ref mut branch) => &mut branch.hint,
        }
    }

    fn to_orphan<'a>(&'a self)
                     -> (OrphanNode<&'a LeafPayload, &'a BranchPayload>, Option<&'a NodeLink>) {
        match *self {
            Node::Leaf(ref leaf) => (OrphanNode::Leaf(&leaf.payload), None),
            Node::Branch(ref branch) => (OrphanNode::Branch(&branch.payload), Some(&branch.nodes)),
        }
    }

    fn to_orphan_mut<'a>
        (&'a mut self)
         -> (OrphanNode<&'a mut LeafPayload, &'a mut BranchPayload>, Option<&'a mut NodeLink>) {
        match *self {
            Node::Leaf(ref mut leaf) => (OrphanNode::Leaf(&mut leaf.payload), None),
            Node::Branch(ref mut branch) => {
                (OrphanNode::Branch(&mut branch.payload), Some(&mut branch.nodes))
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
            *self = Node::Branch(BranchNode::new(Box::new([self.clone(),
                                                           self.clone(),
                                                           self.clone(),
                                                           self.clone(),
                                                           self.clone(),
                                                           self.clone(),
                                                           self.clone(),
                                                           self.clone()])));
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
            Node::Leaf(ref leaf) => Node::Leaf(leaf.clone()),
            Node::Branch(ref branch) => Node::Branch(branch.clone()),
        }
    }
}

pub enum OrphanNode<L, B>
    where L: AsRef<LeafPayload>,
          B: AsRef<BranchPayload>
{
    Leaf(L),
    Branch(B),
}

impl<L, B> OrphanNode<L, B>
    where L: AsRef<LeafPayload>,
          B: AsRef<BranchPayload>
{
    pub fn is_leaf(&self) -> bool {
        match *self {
            OrphanNode::Leaf(..) => true,
            _ => false,
        }
    }

    pub fn as_leaf(&self) -> Option<&LeafPayload> {
        match *self {
            OrphanNode::Leaf(ref leaf) => Some(leaf.as_ref()),
            _ => None,
        }
    }

    pub fn as_branch(&self) -> Option<&BranchPayload> {
        match *self {
            OrphanNode::Branch(ref branch) => Some(branch.as_ref()),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn hint(&self) -> &Hint {
        match *self {
            OrphanNode::Leaf(ref leaf) => &leaf.as_ref().hint,
            OrphanNode::Branch(ref branch) => &branch.as_ref().hint,
        }
    }
}

impl<L, B> OrphanNode<L, B>
    where L: AsRef<LeafPayload> + AsMut<LeafPayload>,
          B: AsRef<BranchPayload> + AsMut<BranchPayload>
{
    pub fn as_leaf_mut(&mut self) -> Option<&mut LeafPayload> {
        match *self {
            OrphanNode::Leaf(ref mut leaf) => Some(leaf.as_mut()),
            _ => None,
        }
    }

    pub fn as_branch_mut(&mut self) -> Option<&mut BranchPayload> {
        match *self {
            OrphanNode::Branch(ref mut branch) => Some(branch.as_mut()),
            _ => None,
        }
    }

    fn hint_mut(&mut self) -> &mut Hint {
        match *self {
            OrphanNode::Leaf(ref mut leaf) => &mut leaf.as_mut().hint,
            OrphanNode::Branch(ref mut branch) => &mut branch.as_mut().hint,
        }
    }
}

#[derive(Clone)]
pub struct LeafNode {
    pub payload: LeafPayload,
}

impl LeafNode {
    fn new() -> Self {
        LeafNode { payload: LeafPayload::new() }
    }
}

impl ops::Deref for LeafNode {
    type Target = LeafPayload;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl ops::DerefMut for LeafNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

#[derive(Clone, Copy)]
pub struct LeafPayload {
    pub geometry: Geometry,
    pub material: ResourceId,
    hint: Hint,
}

impl LeafPayload {
    fn new() -> Self {
        LeafPayload {
            geometry: Geometry::full(),
            material: 0,
            hint: Hint::new(),
        }
    }
}

impl AsRef<LeafPayload> for LeafPayload {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<LeafPayload> for LeafPayload {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub struct BranchNode {
    pub payload: BranchPayload,
    nodes: NodeLink,
}

impl BranchNode {
    fn new(nodes: NodeLink) -> Self {
        BranchNode {
            payload: BranchPayload::new(),
            nodes: nodes,
        }
    }
}

impl Clone for BranchNode {
    fn clone(&self) -> Self {
        BranchNode::new(Box::new([self.nodes[0].clone(),
                                  self.nodes[1].clone(),
                                  self.nodes[2].clone(),
                                  self.nodes[3].clone(),
                                  self.nodes[4].clone(),
                                  self.nodes[5].clone(),
                                  self.nodes[6].clone(),
                                  self.nodes[7].clone()]))
    }
}

impl ops::Deref for BranchNode {
    type Target = BranchPayload;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}

impl ops::DerefMut for BranchNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload
    }
}

#[derive(Clone, Copy)]
pub struct BranchPayload {
    hint: Hint,
}

impl BranchPayload {
    fn new() -> Self {
        BranchPayload { hint: Hint::new() }
    }
}

impl AsRef<BranchPayload> for BranchPayload {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<BranchPayload> for BranchPayload {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone, Copy)]
struct Hint {
    pub load: usize,
}

impl Hint {
    fn new() -> Self {
        Hint { load: 0 }
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
            partition: Partition::at_point(&UPoint3::origin(), width),
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

    pub fn to_orphan(&self) -> OrphanCube<&LeafPayload, &BranchPayload> {
        let (orphan, _) = self.node.as_ref().to_orphan();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn for_each<F>(&self, mut f: F)
        where F: FnMut(&Cube<&Node>)
    {
        traverse!(cube => self.to_value(), |traversal| {
            f(traversal.peek());
            traversal.push();
        });
    }

    pub fn for_each_path<F>(&mut self, mut f: F)
        where F: FnMut((&Cube<&Node>, &[OrphanCube<&LeafPayload, &BranchPayload>]))
    {
        trace!(cube => self.to_value(), |trace| {
            f(trace.peek());
            trace.push();
        });
    }

    pub fn at_point(&self, point: &UPoint3, width: LogWidth) -> Option<Cube<&Node>> {
        if self.partition.aabb().intersects(point) {
            let mut node = self.node.as_ref();
            let mut depth = self.partition.width();

            let point = point.clamp(0, self.root.width().exp() - 1);
            let width = width.clamp(LogWidth::min_value(), depth);
            while width < depth {
                if let Some(branch) = node.as_branch() {
                    depth = depth - 1;
                    node = &branch.nodes[space::index_at_point(&point, depth)]
                }
                else {
                    break;
                }
            }
            Some(Cube::new(node, self.root, Partition::at_point(&point, depth)))
        }
        else {
            None
        }
    }

    pub fn at_index(&self, index: usize) -> Option<Cube<&Node>> {
        self.node.as_ref().as_branch().map_or(None, |branch| {
            self.partition
                .at_index(index)
                .map(|partition| Cube::new(&branch.nodes[index], self.root, partition))
        })
    }

    pub fn at_ray(&self, ray: &FRay3, width: LogWidth) -> Option<(RayIntersection, Cube<&Node>)> {
        let mut min_distance = FScalar::max_value();
        let mut cube = None;
        traverse!(cube => self.to_value(), |traversal| {
            if let Some(intersection) = traversal.peek().aabb().ray_intersection(ray) {
                if traversal.peek().partition.width() >= width {
                    if !traversal.peek().is_empty() { // Non-empty leaf.
                        if intersection.distance < min_distance {
                            min_distance = intersection.distance;
                            // No need to `push`; this is a leaf.
                            cube = Some((intersection, traversal.take()));
                        }
                    }
                    else if traversal.peek().partition.width() > width {
                        traversal.push();
                    }
                }
            }
        });
        cube
    }

    pub fn is_empty(&self) -> bool {
        match *self.node.as_ref() {
            Node::Leaf(ref leaf) => leaf.geometry.is_empty(),
            Node::Branch(..) => true,
        }
    }

    fn to_value(&self) -> Cube<&Node> {
        Cube::new(self.node.as_ref(), self.root, self.partition)
    }
}

impl<'a, 'b, N> Cube<'a, &'b N>
    where N: AsRef<Node>
{
    pub fn into_subdivisions(self) -> (Cube<'a, &'b N>, Option<Vec<Cube<'a, &'b Node>>>) {
        let root = self.root;
        let partition = self.partition;
        let (_, nodes) = self.node.as_ref().to_orphan();
        (self,
         nodes.map(|nodes| {
             let mut cubes = Vec::with_capacity(8);
             for (index, node) in nodes.iter().enumerate() {
                 cubes.push(Cube::new(node, root, partition.at_index(index).unwrap()));
             }
             cubes
         }))
    }

    pub fn into_orphan(self) -> OrphanCube<'a, &'b LeafPayload, &'b BranchPayload> {
        let (orphan, _) = self.node.as_ref().to_orphan();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn iter(&self) -> CubeIter<&N> {
        CubeIter(vec![Cube::new(self.node, self.root, self.partition)])
    }

    pub fn iter_cursor(&self, cursor: &'b Cursor) -> CursorIter<&N> {
        CursorIter {
            cubes: vec![Cube::new(self.node, self.root, self.partition)],
            cursor: cursor,
        }
    }
}

impl<'a, N> Cube<'a, N>
    where N: AsRef<Node> + AsMut<Node>
{
    pub fn to_orphan_mut(&mut self) -> OrphanCube<&mut LeafPayload, &mut BranchPayload> {
        let (orphan, _) = self.node.as_mut().to_orphan_mut();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn for_each_mut<F>(&mut self, mut f: F)
        where F: FnMut(&mut Cube<&mut Node>)
    {
        traverse!(cube => self.to_value_mut(), |traversal| {
            f(traversal.peek_mut());
            traversal.push();
        });
    }

    pub fn for_each_path_mut<F>(&mut self, mut f: F)
        where F: FnMut((&mut Cube<&mut Node>,
                        &mut [OrphanCube<&mut LeafPayload, &mut BranchPayload>]))
    {
        trace!(cube => self.to_value_mut(), |trace| {
            f(trace.peek_mut());
            trace.push();
        });
    }

    pub fn at_point_mut(&mut self, point: &UPoint3, width: LogWidth) -> Option<Cube<&mut Node>> {
        self.for_each_node_to_point(point, width, |_| {})
    }

    pub fn at_index_mut(&mut self, index: usize) -> Option<Cube<&mut Node>> {
        match *self.node.as_mut() {
            Node::Branch(ref mut branch) => {
                let root = self.root;
                self.partition
                    .at_index(index)
                    .map(move |partition| Cube::new(&mut branch.nodes[index], root, partition))
            }
            _ => None,
        }
    }

    pub fn at_ray_mut(&mut self,
                      ray: &FRay3,
                      width: LogWidth)
                      -> Option<(RayIntersection, Cube<&mut Node>)> {
        let mut min_distance = FScalar::max_value();
        let mut cube = None;
        traverse!(cube => self.to_value_mut(), |traversal| {
            if let Some(intersection) = traversal.peek().aabb().ray_intersection(ray) {
                if traversal.peek().partition.width() >= width {
                    if !traversal.peek().is_empty() { // Non-empty leaf.
                        if intersection.distance < min_distance {
                            min_distance = intersection.distance;
                            // No need to `push`; this is a leaf.
                            cube = Some((intersection, traversal.take()));
                        }
                    }
                    else if traversal.peek().partition.width() > width {
                        traversal.push();
                    }
                }
            }
        });
        cube
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

    pub fn subdivide_to_point(&mut self,
                              point: &UPoint3,
                              width: LogWidth)
                              -> Option<Cube<&mut Node>> {
        self.for_each_node_to_point(point, width, |node| {
            let _ = node.subdivide();
        })
    }

    pub fn subdivide_to_cursor(&mut self, cursor: &Cursor) -> Vec<Cube<&mut Node>> {
        let mut cubes = vec![];
        traverse!(cube => self.to_value_mut(), |traversal| {
            if traversal.peek().aabb().intersects(&cursor.aabb()) {
                if traversal.peek().partition.width() == cursor.width() {
                    cubes.push(traversal.take());
                }
                else if traversal.peek().partition.width() > cursor.width() {
                    let _ = traversal.peek_mut().node.as_mut().subdivide();
                    traversal.push();
                }
            }
        });
        cubes
    }

    #[allow(dead_code)]
    fn instrument(&mut self) -> usize {
        self.for_each_path_mut(|(cube, path)| {
            if cube.is_leaf() {
                cube.hint_mut().load = 0;
            }
            else {
                for cube in path.iter_mut() {
                    cube.hint_mut().load += 1;
                }
                cube.hint_mut().load = 1;
            }
        });
        self.hint().load
    }

    fn for_each_node_to_point<F>(&mut self,
                                 point: &UPoint3,
                                 width: LogWidth,
                                 mut f: F)
                                 -> Option<Cube<&mut Node>>
        where F: FnMut(&mut Node)
    {
        if self.partition.aabb().intersects(point) {
            let mut node: Option<&mut Node> = Some(self.node.as_mut());
            let mut depth = self.partition.width();

            let point = point.clamp(0, self.root.width().exp() - 1);
            let width = width.clamp(LogWidth::min_value(), depth);
            while width < depth {
                let taken = node.take().unwrap();
                f(taken);
                match *taken {
                    Node::Branch(ref mut branch) => {
                        depth = depth - 1;
                        node = Some(&mut branch.nodes[space::index_at_point(&point, depth)]);
                    }
                    _ => {
                        node = Some(taken);
                        break;
                    }
                }
            }
            Some(Cube::new(node.take().unwrap(),
                           self.root,
                           Partition::at_point(&point, depth)))
        }
        else {
            None
        }
    }

    fn to_value_mut(&mut self) -> Cube<&mut Node> {
        Cube::new(self.node.as_mut(), self.root, self.partition)
    }
}

impl<'a, 'b, N> Cube<'a, &'b mut N>
    where N: AsRef<Node> + AsMut<Node>
{
    pub fn into_subdivisions_mut(self)
                                 -> (OrphanCube<'a, &'b mut LeafPayload, &'b mut BranchPayload>,
                                     Option<Vec<Cube<'a, &'b mut Node>>>)
    {
        let root = self.root;
        let partition = self.partition;
        let (orphan, nodes) = self.node.as_mut().to_orphan_mut();
        (OrphanCube::new(orphan, root, partition),
         nodes.map(|nodes| {
             let mut cubes = Vec::with_capacity(8);
             for (index, node) in nodes.iter_mut().enumerate() {
                 cubes.push(Cube::new(node, root, partition.at_index(index).unwrap()));
             }
             cubes
         }))
    }

    pub fn into_orphan_mut(self) -> OrphanCube<'a, &'b mut LeafPayload, &'b mut BranchPayload> {
        let (orphan, _) = self.node.as_mut().to_orphan_mut();
        OrphanCube::new(orphan, self.root, self.partition)
    }

    pub fn iter_mut(&mut self) -> CubeIter<&mut N> {
        CubeIter(vec![Cube::new(self.node, self.root, self.partition)])
    }

    pub fn iter_cursor_mut(&mut self, cursor: &'b Cursor) -> CursorIter<&mut N> {
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
        self.root.width().to_inner() - self.partition.width().to_inner()
    }
}

pub struct CubeIter<'a, N>(Vec<Cube<'a, N>>) where N: AsRef<Node>;

impl<'a> Iterator for CubeIter<'a, &'a Node> {
    type Item = Cube<'a, &'a Node>;

    fn next(&mut self) -> Option<Self::Item> {
        traverse!(buffer => self.0, |traversal| {
            return Some(traversal.push());
        });
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(cube) = self.0.last() {
            (1 + (8 * cube.hint().load), None)
        }
        else {
            (0, None)
        }
    }
}

impl<'a> Iterator for CubeIter<'a, &'a mut Node> {
    type Item = OrphanCube<'a, &'a mut LeafPayload, &'a mut BranchPayload>;

    fn next(&mut self) -> Option<Self::Item> {
        traverse!(buffer => self.0, |traversal| {
            return Some(traversal.push());
        });
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(cube) = self.0.last() {
            (1 + (8 * cube.hint().load), None)
        }
        else {
            (0, None)
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
        traverse!(buffer => self.cubes, |traversal| {
            if traversal.peek().aabb().intersects(&self.cursor.aabb()) {
                if traversal.peek().partition.width() == self.cursor.width() {
                    return Some(traversal.take());
                }
                else if traversal.peek().partition.width() > self.cursor.width() {
                    traversal.push();
                }
            }
        });
        None
    }
}

impl<'a> Iterator for CursorIter<'a, &'a mut Node> {
    type Item = Cube<'a, &'a mut Node>;

    fn next(&mut self) -> Option<Self::Item> {
        traverse!(buffer => self.cubes, |traversal| {
            if traversal.peek().aabb().intersects(&self.cursor.aabb()) {
                if traversal.peek().partition.width() == self.cursor.width() {
                    return Some(traversal.take());
                }
                else if traversal.peek().partition.width() > self.cursor.width() {
                    traversal.push();
                }
            }
        });
        None
    }
}

pub struct OrphanCube<'a, L, B>
    where L: AsRef<LeafPayload>,
          B: AsRef<BranchPayload>
{
    node: OrphanNode<L, B>,
    root: &'a Partition,
    partition: Partition,
}

impl<'a, L, B> OrphanCube<'a, L, B>
    where L: AsRef<LeafPayload>,
          B: AsRef<BranchPayload>
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
    where L: AsRef<LeafPayload>,
          B: AsRef<BranchPayload>
{
    type Target = OrphanNode<L, B>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, L, B> ops::DerefMut for OrphanCube<'a, L, B>
    where L: AsRef<LeafPayload> + AsMut<LeafPayload>,
          B: AsRef<BranchPayload> + AsMut<BranchPayload>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl<'a, L, B> Spatial for OrphanCube<'a, L, B>
    where L: AsRef<LeafPayload>,
          B: AsRef<BranchPayload>
{
    fn partition(&self) -> &Partition {
        &self.partition
    }

    fn depth(&self) -> u8 {
        self.root.width().to_inner() - self.partition.width().to_inner()
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
