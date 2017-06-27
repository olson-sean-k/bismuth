use arrayvec::{Array, ArrayVec};
use std::collections::VecDeque;
use std::iter::IntoIterator;

use math::{self, FScalar};
use super::primitive::{Line, Polygon, Polygonal, Primitive, Triangle, Quad};

// A type `F` constrained to `Fn(P, D) -> R` could be used here, but it would
// not be possible to name that type for anything but functions (not closures).
// Instead of a limited and somewhat redundant type `F`, just use `fn(P, D) ->
// R` for the member `f`.
pub struct Decompose<I, P, Q, D, R>
where
    D: Copy,
    R: IntoIterator<Item = Q>,
{
    input: I,
    output: VecDeque<Q>,
    parameter: D,
    f: fn(P, D) -> R,
}

impl<I, P, Q, D, R> Decompose<I, P, Q, D, R>
where
    D: Copy,
    R: IntoIterator<Item = Q>,
{
    pub(super) fn new(input: I, parameter: D, f: fn(P, D) -> R) -> Self {
        Decompose {
            input: input,
            output: VecDeque::new(),
            parameter: parameter,
            f: f,
        }
    }
}

impl<I, P, Q, D, R> Iterator for Decompose<I, P, Q, D, R>
where
    I: Iterator<Item = P>,
    D: Copy,
    R: IntoIterator<Item = Q>,
{
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(polygon) = self.output.pop_front() {
                return Some(polygon);
            }
            if let Some(polygon) = self.input.next() {
                self.output.extend((self.f)(polygon, self.parameter));
            }
            else {
                return None;
            }
        }
    }
}

pub trait Interpolate: math::Interpolate<FScalar> {}

impl<T> Interpolate for T
where
    T: math::Interpolate<FScalar>,
{
}

pub trait IntoPoints: Primitive {
    type Output: IntoIterator<Item = Self::Point>;

    fn into_points(self) -> Self::Output;
}

pub trait IntoLines: Primitive {
    type Output: IntoIterator<Item = Line<Self::Point>>;

    fn into_lines(self) -> Self::Output;
}

pub trait IntoTriangles: Polygonal {
    type Output: IntoIterator<Item = Triangle<Self::Point>>;

    fn into_triangles(self) -> Self::Output;
}

pub trait IntoSubdivisions: Polygonal
where
    Self::Point: Clone + Interpolate,
{
    fn into_subdivisions(self, n: usize) -> Vec<Self>;
}

pub trait IntoTetrahedrons: Polygonal
where
    Self::Point: Clone + Interpolate,
{
    fn into_tetrahedrons(self) -> ArrayVec<[Triangle<Self::Point>; 4]>;
}

impl<T> IntoPoints for Line<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Self::Point; 2]>;

    fn into_points(self) -> Self::Output {
        let Line { a, b } = self;
        ArrayVec::from([a, b])
    }
}

impl<T> IntoPoints for Triangle<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Self::Point; 3]>;

    fn into_points(self) -> Self::Output {
        let Triangle { a, b, c } = self;
        ArrayVec::from([a, b, c])
    }
}

impl<T> IntoPoints for Quad<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Self::Point; 4]>;

    fn into_points(self) -> Self::Output {
        let Quad { a, b, c, d } = self;
        ArrayVec::from([a, b, c, d])
    }
}

impl<T> IntoPoints for Polygon<T>
where
    T: Clone,
{
    type Output = Vec<Self::Point>;

    fn into_points(self) -> Self::Output {
        match self {
            Polygon::Triangle(triangle) => triangle.into_points().into_iter().collect(),
            Polygon::Quad(quad) => quad.into_points().into_iter().collect(),
        }
    }
}

impl<T> IntoLines for Line<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Line<Self::Point>; 1]>;

    fn into_lines(self) -> Self::Output {
        ArrayVec::from([self])
    }
}

impl<T> IntoLines for Triangle<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Line<Self::Point>; 3]>;

    fn into_lines(self) -> Self::Output {
        let Triangle { a, b, c } = self;
        ArrayVec::from(
            [
                Line::new(a.clone(), b.clone()),
                Line::new(b, c.clone()),
                Line::new(c, a),
            ],
        )
    }
}

impl<T> IntoLines for Quad<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Line<Self::Point>; 4]>;

    fn into_lines(self) -> Self::Output {
        let Quad { a, b, c, d } = self;
        ArrayVec::from(
            [
                Line::new(a.clone(), b.clone()),
                Line::new(b, c.clone()),
                Line::new(c, d.clone()),
                Line::new(d, a),
            ],
        )
    }
}

impl<T> IntoLines for Polygon<T>
where
    T: Clone,
{
    type Output = Vec<Line<Self::Point>>;

    fn into_lines(self) -> Self::Output {
        match self {
            Polygon::Triangle(triangle) => triangle.into_lines().into_iter().collect(),
            Polygon::Quad(quad) => quad.into_lines().into_iter().collect(),
        }
    }
}

impl<T> IntoTriangles for Triangle<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Triangle<Self::Point>; 1]>;

    fn into_triangles(self) -> Self::Output {
        ArrayVec::from([self])
    }
}

impl<T> IntoTriangles for Quad<T>
where
    T: Clone,
{
    type Output = ArrayVec<[Triangle<Self::Point>; 2]>;

    fn into_triangles(self) -> Self::Output {
        let Quad { a, b, c, d } = self;
        ArrayVec::from(
            [
                Triangle::new(a.clone(), b, c.clone()),
                Triangle::new(c, d, a),
            ],
        )
    }
}

impl<T> IntoTriangles for Polygon<T>
where
    T: Clone,
{
    type Output = Vec<Triangle<Self::Point>>;

    fn into_triangles(self) -> Self::Output {
        match self {
            Polygon::Triangle(triangle) => triangle.into_triangles().into_iter().collect(),
            Polygon::Quad(quad) => quad.into_triangles().into_iter().collect(),
        }
    }
}

impl<T> IntoSubdivisions for Triangle<T>
where
    T: Clone + Interpolate,
{
    fn into_subdivisions(self, n: usize) -> Vec<Self> {
        n_map_polygon(n, self, |triangle| {
            let Triangle { a, b, c } = triangle;
            let ac = a.midpoint(&c);
            ArrayVec::from(
                [
                    Triangle::new(b.clone(), ac.clone(), a),
                    Triangle::new(c, ac, b),
                ],
            )
        })
    }
}

impl<T> IntoSubdivisions for Quad<T>
where
    T: Clone + Interpolate,
{
    fn into_subdivisions(self, n: usize) -> Vec<Self> {
        n_map_polygon(n, self, |quad| {
            let Quad { a, b, c, d } = quad;
            let ab = a.midpoint(&b);
            let bc = b.midpoint(&c);
            let cd = c.midpoint(&d);
            let da = d.midpoint(&a);
            let ac = a.midpoint(&c); // Diagonal.
            ArrayVec::from(
                [
                    Quad::new(a, ab.clone(), ac.clone(), da.clone()),
                    Quad::new(ab, b, bc.clone(), ac.clone()),
                    Quad::new(ac.clone(), bc, c, cd.clone()),
                    Quad::new(da, ac, cd, d),
                ],
            )
        })
    }
}

impl<T> IntoTetrahedrons for Quad<T>
where
    T: Clone + Interpolate,
{
    fn into_tetrahedrons(self) -> ArrayVec<[Triangle<Self::Point>; 4]> {
        let Quad { a, b, c, d } = self;
        let ac = a.midpoint(&c); // Diagonal.
        ArrayVec::from(
            [
                Triangle::new(a.clone(), b.clone(), ac.clone()),
                Triangle::new(b, c.clone(), ac.clone()),
                Triangle::new(c, d.clone(), ac.clone()),
                Triangle::new(d, a, ac),
            ],
        )
    }
}

impl<T> IntoSubdivisions for Polygon<T>
where
    T: Clone + Interpolate,
{
    fn into_subdivisions(self, n: usize) -> Vec<Self> {
        match self {
            Polygon::Triangle(triangle) => {
                triangle
                    .into_subdivisions(n)
                    .into_iter()
                    .map(|triangle| triangle.into())
                    .collect()
            }
            Polygon::Quad(quad) => {
                quad.into_subdivisions(n)
                    .into_iter()
                    .map(|quad| quad.into())
                    .collect()
            }
        }
    }
}

pub trait Points<P>: Sized
where
    P: IntoPoints,
{
    fn points(self) -> Decompose<Self, P, P::Point, (), P::Output>;
}

impl<I, P> Points<P> for I
where
    I: Iterator<Item = P>,
    P: IntoPoints,
    P::Point: Clone,
{
    fn points(self) -> Decompose<Self, P, P::Point, (), P::Output> {
        Decompose::new(self, (), into_points)
    }
}

pub trait Triangulate<P>: Sized
where
    P: IntoTriangles,
{
    fn triangulate(self) -> Decompose<Self, P, Triangle<P::Point>, (), P::Output>;
}

impl<I, P> Triangulate<P> for I
where
    I: Iterator<Item = P>,
    P: IntoTriangles,
    P::Point: Clone,
{
    fn triangulate(self) -> Decompose<Self, P, Triangle<P::Point>, (), P::Output> {
        Decompose::new(self, (), into_triangles)
    }
}

pub trait Subdivide<P>: Sized
where
    P: IntoSubdivisions,
    P::Point: Clone + Interpolate,
{
    fn subdivide(self, n: usize) -> Decompose<Self, P, P, usize, Vec<P>>;
}

impl<I, P> Subdivide<P> for I
where
    I: Iterator<Item = P>,
    P: IntoSubdivisions,
    P::Point: Clone + Interpolate,
{
    fn subdivide(self, n: usize) -> Decompose<Self, P, P, usize, Vec<P>> {
        Decompose::new(self, n, P::into_subdivisions)
    }
}

pub trait Tetrahedrons<T>: Sized {
    #[allow(type_complexity)]
    fn tetrahedrons(self) -> Decompose<Self, Quad<T>, Triangle<T>, (), ArrayVec<[Triangle<T>; 4]>>;
}

impl<I, T> Tetrahedrons<T> for I
where
    I: Iterator<Item = Quad<T>>,
    T: Clone + Interpolate,
{
    #[allow(type_complexity)]
    fn tetrahedrons(self) -> Decompose<Self, Quad<T>, Triangle<T>, (), ArrayVec<[Triangle<T>; 4]>> {
        Decompose::new(self, (), into_tetrahedrons)
    }
}

fn into_points<P>(primitive: P, _: ()) -> P::Output
where
    P: IntoPoints,
    P::Point: Clone,
{
    primitive.into_points()
}

fn into_triangles<P>(polygon: P, _: ()) -> P::Output
where
    P: IntoTriangles,
    P::Point: Clone,
{
    polygon.into_triangles()
}

fn into_tetrahedrons<T>(quad: Quad<T>, _: ()) -> ArrayVec<[Triangle<T>; 4]>
where
    T: Clone + Interpolate,
{
    quad.into_tetrahedrons()
}

fn n_map_polygon<P, A, F>(n: usize, polygon: P, f: F) -> Vec<P>
where
    P: Polygonal,
    P::Point: Clone,
    A: Array<Item = P>,
    F: Fn(P) -> ArrayVec<A>,
{
    let mut polygons = vec![polygon];
    for _ in 0..n {
        polygons = polygons.into_iter().flat_map(&f).collect();
    }
    polygons
}
