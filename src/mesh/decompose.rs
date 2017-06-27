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
    fn into_points(self) -> Vec<Self::Point>;
}

impl<T> IntoPoints for Line<T>
where
    T: Clone,
{
    fn into_points(self) -> Vec<Self::Point> {
        let Line { a, b } = self;
        vec![a, b]
    }
}

impl<T> IntoPoints for Triangle<T>
where
    T: Clone,
{
    fn into_points(self) -> Vec<Self::Point> {
        let Triangle { a, b, c } = self;
        vec![a, b, c]
    }
}

impl<T> IntoPoints for Quad<T>
where
    T: Clone,
{
    fn into_points(self) -> Vec<Self::Point> {
        let Quad { a, b, c, d } = self;
        vec![a, b, c, d]
    }
}

impl<T> IntoPoints for Polygon<T>
where
    T: Clone,
{
    fn into_points(self) -> Vec<Self::Point> {
        match self {
            Polygon::Triangle(triangle) => triangle.into_points(),
            Polygon::Quad(quad) => quad.into_points(),
        }
    }
}

pub trait IntoLines: Primitive {
    fn into_lines(self) -> Vec<Line<Self::Point>>;
}

impl<T> IntoLines for Line<T>
where
    T: Clone,
{
    fn into_lines(self) -> Vec<Line<Self::Point>> {
        vec![self]
    }
}

impl<T> IntoLines for Triangle<T>
where
    T: Clone,
{
    fn into_lines(self) -> Vec<Line<Self::Point>> {
        let Triangle { a, b, c } = self;
        vec![
            Line::new(a.clone(), b.clone()),
            Line::new(b, c.clone()),
            Line::new(c, a),
        ]
    }
}

impl<T> IntoLines for Quad<T>
where
    T: Clone,
{
    fn into_lines(self) -> Vec<Line<Self::Point>> {
        let Quad { a, b, c, d } = self;
        vec![
            Line::new(a.clone(), b.clone()),
            Line::new(b, c.clone()),
            Line::new(c, d.clone()),
            Line::new(d, a),
        ]
    }
}

impl<T> IntoLines for Polygon<T>
where
    T: Clone,
{
    fn into_lines(self) -> Vec<Line<Self::Point>> {
        match self {
            Polygon::Triangle(triangle) => triangle.into_lines(),
            Polygon::Quad(quad) => quad.into_lines(),
        }
    }
}

pub trait IntoTriangles: Polygonal {
    fn into_triangles(self) -> Vec<Triangle<Self::Point>>;
}

impl<T> IntoTriangles for Triangle<T>
where
    T: Clone,
{
    fn into_triangles(self) -> Vec<Triangle<Self::Point>> {
        vec![self]
    }
}

impl<T> IntoTriangles for Quad<T>
where
    T: Clone,
{
    fn into_triangles(self) -> Vec<Triangle<Self::Point>> {
        let Quad { a, b, c, d } = self;
        vec![
            Triangle::new(a.clone(), b, c.clone()),
            Triangle::new(c, d, a),
        ]
    }
}

impl<T> IntoTriangles for Polygon<T>
where
    T: Clone,
{
    fn into_triangles(self) -> Vec<Triangle<Self::Point>> {
        match self {
            Polygon::Triangle(triangle) => triangle.into_triangles(),
            Polygon::Quad(quad) => quad.into_triangles(),
        }
    }
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
    P: Primitive,
{
    fn points(self) -> Decompose<Self, P, P::Point, (), Vec<P::Point>>;
}

impl<I, T, P> Points<P> for I
where
    I: Iterator<Item = P>,
    P: IntoPoints<Point = T>,
    T: Clone,
{
    fn points(self) -> Decompose<Self, P, P::Point, (), Vec<P::Point>> {
        Decompose::new(self, (), into_points)
    }
}

pub trait Triangulate<P>: Sized
where
    P: Polygonal,
{
    #[allow(type_complexity)]
    fn triangulate(self) -> Decompose<Self, P, Triangle<P::Point>, (), Vec<Triangle<P::Point>>>;
}

impl<I, T, P> Triangulate<P> for I
where
    I: Iterator<Item = P>,
    P: IntoTriangles<Point = T>,
    T: Clone,
{
    #[allow(type_complexity)]
    fn triangulate(self) -> Decompose<Self, P, Triangle<P::Point>, (), Vec<Triangle<P::Point>>> {
        Decompose::new(self, (), into_triangles)
    }
}

pub trait Subdivide<P>: Sized
where
    P: Polygonal,
{
    fn subdivide(self, n: usize) -> Decompose<Self, P, P, usize, Vec<P>>;
}

impl<I, T, P> Subdivide<P> for I
where
    I: Iterator<Item = P>,
    P: IntoSubdivisions<Point = T>,
    T: Clone + Interpolate,
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

fn into_points<P, T>(primitive: P, _: ()) -> Vec<T>
where
    P: IntoPoints<Point = T>,
    T: Clone,
{
    primitive.into_points()
}

fn into_triangles<P, T>(polygon: P, _: ()) -> Vec<Triangle<T>>
where
    P: IntoTriangles<Point = T>,
    T: Clone,
{
    polygon.into_triangles()
}

fn into_tetrahedrons<T>(quad: Quad<T>, _: ()) -> ArrayVec<[Triangle<T>; 4]>
where
    T: Clone + Interpolate,
{
    quad.into_tetrahedrons()
}

fn n_map_polygon<T, P, A, F>(n: usize, polygon: P, f: F) -> Vec<P>
where
    P: Polygonal<Point = T>,
    T: Clone,
    A: Array<Item = P>,
    F: Fn(P) -> ArrayVec<A>,
{
    let mut polygons = vec![polygon];
    for _ in 0..n {
        polygons = polygons.into_iter().flat_map(&f).collect();
    }
    polygons
}
