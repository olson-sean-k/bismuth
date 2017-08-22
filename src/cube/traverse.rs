use cube::tree::{BranchPayload, Cube, LeafPayload, Node, OrphanCube};

pub trait TraversalBuffer<'a, N>: Extend<Cube<'a, N>>
where
    N: AsRef<Node>,
{
    fn pop(&mut self) -> Option<Cube<'a, N>>;
    fn push(&mut self, cube: Cube<'a, N>);
}

impl<'a, N> TraversalBuffer<'a, N> for Vec<Cube<'a, N>>
where
    N: AsRef<Node>,
{
    fn pop(&mut self) -> Option<Cube<'a, N>> {
        self.pop()
    }

    fn push(&mut self, cube: Cube<'a, N>) {
        self.push(cube);
    }
}

pub struct Traversal<'a, 'b, N, B>
where
    N: 'b + AsRef<Node>,
    B: 'b + TraversalBuffer<'b, N>,
    'b: 'a,
{
    cubes: &'a mut B,
    cube: Cube<'b, N>,
}

impl<'a, 'b, N, B> Traversal<'a, 'b, N, B>
where
    N: 'b + AsRef<Node>,
    B: 'b + TraversalBuffer<'b, N>,
    'b: 'a,
{
    // This probably shouldn't be `pub` at all, but because of the use of
    // macros, it must be.
    pub(super) fn new(cubes: &'a mut B, cube: Cube<'b, N>) -> Self {
        Traversal {
            cubes: cubes,
            cube: cube,
        }
    }

    pub fn peek(&self) -> &Cube<'b, N> {
        &self.cube
    }

    pub fn take(self) -> Cube<'b, N> {
        self.cube
    }
}

impl<'a, 'b, N, B> Traversal<'a, 'b, N, B>
where
    N: 'b + AsRef<Node> + AsMut<Node>,
    B: 'b + TraversalBuffer<'b, N>,
    'b: 'a,
{
    pub fn peek_mut(&mut self) -> &mut Cube<'b, N> {
        &mut self.cube
    }
}

impl<'a, 'b, 'c, B> Traversal<'a, 'b, &'c Node, B>
where
    B: 'b + TraversalBuffer<'b, &'c Node>,
{
    pub fn push(self) -> Cube<'b, &'c Node> {
        let (cube, cubes) = self.cube.into_subdivisions();
        if let Some(cubes) = cubes {
            self.cubes.extend(cubes);
        }
        cube
    }
}

impl<'a, 'b, 'c, B> Traversal<'a, 'b, &'c mut Node, B>
where
    B: 'b + TraversalBuffer<'b, &'c mut Node>,
{
    pub fn push(self) -> OrphanCube<'b, &'c mut LeafPayload, &'c mut BranchPayload> {
        let (orphan, cubes) = self.cube.into_subdivisions_mut();
        if let Some(cubes) = cubes {
            self.cubes.extend(cubes);
        }
        orphan
    }
}

pub struct PathTraversal<'a, 'b, N, L, B, T>
where
    N: 'b + AsRef<Node>,
    L: 'b + AsRef<LeafPayload>,
    B: 'b + AsRef<BranchPayload>,
    T: 'b + TraversalBuffer<'b, N>,
    'b: 'a,
{
    traversal: Traversal<'a, 'b, N, T>,
    path: &'a mut Vec<OrphanCube<'b, L, B>>,
}

impl<'a, 'b, N, L, B, T> PathTraversal<'a, 'b, N, L, B, T>
where
    N: 'b + AsRef<Node>,
    L: 'b + AsRef<LeafPayload>,
    B: 'b + AsRef<BranchPayload>,
    T: 'b + TraversalBuffer<'b, N>,
    'b: 'a,
{
    // This probably shouldn't be `pub` at all, but because of the use of
    // macros, it must be.
    pub(super) fn new(
        traversal: Traversal<'a, 'b, N, T>,
        path: &'a mut Vec<OrphanCube<'b, L, B>>,
    ) -> Self {
        PathTraversal {
            traversal: traversal,
            path: path,
        }
    }

    pub fn peek(&self) -> (&Cube<'b, N>, &[OrphanCube<'b, L, B>]) {
        (self.traversal.peek(), self.path.as_slice())
    }

    #[allow(dead_code)]
    pub fn take(self) -> Cube<'b, N> {
        self.traversal.take()
    }
}

impl<'a, 'b, N, L, B, T> PathTraversal<'a, 'b, N, L, B, T>
where
    N: 'b + AsRef<Node> + AsMut<Node>,
    L: 'b + AsRef<LeafPayload> + AsMut<LeafPayload>,
    B: 'b + AsRef<BranchPayload> + AsMut<BranchPayload>,
    T: 'b + TraversalBuffer<'b, N>,
    'b: 'a,
{
    pub fn peek_mut(&mut self) -> (&mut Cube<'b, N>, &mut [OrphanCube<'b, L, B>]) {
        (self.traversal.peek_mut(), self.path.as_mut_slice())
    }
}

impl<'a, 'b, 'c, T> PathTraversal<'a, 'b, &'c Node, &'c LeafPayload, &'c BranchPayload, T>
where
    T: 'b + TraversalBuffer<'b, &'c Node>,
{
    pub fn push(self) {
        self.path.push(self.traversal.push().into_orphan());
    }
}

impl<'a, 'b, 'c, T>
    PathTraversal<'a, 'b, &'c mut Node, &'c mut LeafPayload, &'c mut BranchPayload, T>
where
    T: 'b + TraversalBuffer<'b, &'c mut Node>,
{
    pub fn push(self) {
        self.path.push(self.traversal.push());
    }
}

#[macro_export]
macro_rules! traverse {
    (cube => $c:expr, | $t:ident | $f:block) => {{
        let mut cubes = vec![$c];
        traverse!(buffer => cubes, |$t| $f)
    }};
    (buffer => $b:expr, | $t:ident | $f:block) => {{
        #[allow(never_loop)]
        #[allow(unused_mut)]
        while let Some(cube) = $b.pop() {
            let mut $t = Traversal::new(&mut $b, cube);
            $f
        }
    }};
}

#[macro_export]
macro_rules! traverse_with_path {
    (cube => $c:expr, | $t:ident | $f:block) => {{
        let mut path = vec![];
        traverse_with_path!(cube => $c, path => path, |$t| $f)
    }};
    (cube => $c:expr, path => $p:expr, | $t:ident | $f:block) => {{
        let mut depth = $c.depth();
        traverse!(cube => $c, |traversal| {
            if depth > traversal.peek().depth() {
                for _ in 0..(depth - traversal.peek().depth()) {
                    $p.pop();
                }
            }
            depth = traversal.peek().depth();
            let terminal = traversal.peek().is_leaf();
            {
                let mut $t = PathTraversal::new(traversal, &mut $p);
                $f
            }
            if terminal {
                $p.pop();
            }
        });
    }};
}
