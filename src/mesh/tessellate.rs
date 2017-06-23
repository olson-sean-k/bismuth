use std::collections::VecDeque;
use std::iter::IntoIterator;
use std::marker::PhantomData;

use math::{self, FScalar};
use super::primitive::{Polygon, Polygonal, Primitive, Triangle, Quad};

pub struct Tessellate<I, P, Q, D, R, F>
    where D: Copy,
          F: Fn(P, D) -> R,
          R: IntoIterator<Item = Q>
{
    input: I,
    output: VecDeque<Q>,
    parameter: D,
    f: F,
    phantom: PhantomData<(P, R)>,
}

impl<I, P, Q, D, R, F> Tessellate<I, P, Q, D, R, F>
    where D: Copy,
          F: Fn(P, D) -> R,
          R: IntoIterator<Item = Q>
{
    pub(super) fn new(input: I, parameter: D, f: F) -> Self {
        Tessellate {
            input: input,
            output: VecDeque::new(),
            parameter: parameter,
            f: f,
            phantom: PhantomData,
        }
    }
}

impl<I, P, Q, D, R, F> Iterator for Tessellate<I, P, Q, D, R, F>
    where I: Iterator<Item = P>,
          D: Copy,
          F: Fn(P, D) -> R,
          R: IntoIterator<Item = Q>
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

impl<T> Interpolate for T where T: math::Interpolate<FScalar> {}

pub trait IntoSubdivisions: Polygonal
    where Self::Point: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, f: F) where F: FnMut(Self);
}

pub trait IntoTetrahedrons: Polygonal
    where Self::Point: Clone + Interpolate
{
    fn into_tetrahedrons<F>(self, f: F) where F: FnMut(Triangle<Self::Point>);
}

impl<T> IntoSubdivisions for Triangle<T>
    where T: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, mut f: F)
        where F: FnMut(Self)
    {
        for triangle in n_map_polygon(n, self, |triangle| {
            let Triangle { a, b, c } = triangle;
            let ac = a.midpoint(&c);
            vec![Triangle::new(b.clone(), ac.clone(), a),
                 Triangle::new(c, ac, b)]
        }) {
            f(triangle);
        }
    }
}

impl<T> IntoSubdivisions for Quad<T>
    where T: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, mut f: F)
        where F: FnMut(Self)
    {
        for quad in n_map_polygon(n, self, |quad| {
            let Quad { a, b, c, d } = quad;
            let ab = a.midpoint(&b);
            let bc = b.midpoint(&c);
            let cd = c.midpoint(&d);
            let da = d.midpoint(&a);
            let ac = a.midpoint(&c); // Diagonal.
            vec![Quad::new(a, ab.clone(), ac.clone(), da.clone()),
                 Quad::new(ab, b, bc.clone(), ac.clone()),
                 Quad::new(ac.clone(), bc, c, cd.clone()),
                 Quad::new(da, ac, cd, d)]
        }) {
            f(quad);
        }
    }
}

impl<T> IntoTetrahedrons for Quad<T>
    where T: Clone + Interpolate
{
    fn into_tetrahedrons<F>(self, mut f: F)
        where F: FnMut(Triangle<Self::Point>)
    {
        let Quad { a, b, c, d } = self;
        let ac = a.midpoint(&c); // Diagonal.
        f(Triangle::new(a.clone(), b.clone(), ac.clone()));
        f(Triangle::new(b, c.clone(), ac.clone()));
        f(Triangle::new(c, d.clone(), ac.clone()));
        f(Triangle::new(d, a, ac));
    }
}

impl<T> IntoSubdivisions for Polygon<T>
    where T: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, mut f: F)
        where F: FnMut(Self)
    {
        match self {
            Polygon::Triangle(triangle) => {
                triangle.into_subdivisions(n, |triangle| f(triangle.into()));
            }
            Polygon::Quad(quad) => { quad.into_subdivisions(n, |quad| f(quad.into())); }
        }
    }
}

pub trait Points<P>: Sized
    where P: Primitive
{
    fn points(self)
        -> Tessellate<Self, P, P::Point, (), Vec<P::Point>, fn(P, ()) -> Vec<P::Point>>;
}

impl<I, T, P> Points<P> for I
    where I: Iterator<Item = P>,
          P: Primitive<Point = T>,
          T: Clone
{
    fn points(self)
        -> Tessellate<Self, P, P::Point, (), Vec<P::Point>, fn(P, ()) -> Vec<P::Point>>
    {
        Tessellate::new(self, (), into_points)
    }
}

pub trait Triangulate<P>: Sized
    where P: Polygonal
{
    fn triangulate(self)
        -> Tessellate<Self, P, Triangle<P::Point>, (), Vec<Triangle<P::Point>>,
                      fn(P, ()) -> Vec<Triangle<P::Point>>>;
}

impl<I, T, P> Triangulate<P> for I
    where I: Iterator<Item = P>,
          P: Polygonal<Point = T>,
          T: Clone
{
    fn triangulate(self)
        -> Tessellate<Self, P, Triangle<P::Point>, (), Vec<Triangle<P::Point>>,
                      fn(P, ()) -> Vec<Triangle<P::Point>>>
    {
        Tessellate::new(self, (), into_triangles)
    }
}

pub trait Subdivide<P>: Sized
    where P: Polygonal
{
    fn subdivide(self, n: usize) -> Tessellate<Self, P, P, usize, Vec<P>, fn(P, usize) -> Vec<P>>;
}

impl<I, T, P> Subdivide<P> for I
    where I: Iterator<Item = P>,
          P: IntoSubdivisions<Point = T>,
          T: Clone + Interpolate
{
    fn subdivide(self, n: usize) -> Tessellate<Self, P, P, usize, Vec<P>, fn(P, usize) -> Vec<P>> {
        Tessellate::new(self, n, into_subdivisions)
    }
}

pub trait Tetrahedrons<T>: Sized {
    fn tetrahedrons(self)
        -> Tessellate<Self, Quad<T>, Triangle<T>, (), Vec<Triangle<T>>,
                     fn(Quad<T>, ()) -> Vec<Triangle<T>>>;
}

impl<I, T> Tetrahedrons<T> for I
    where I: Iterator<Item = Quad<T>>,
          T: Clone + Interpolate
{
    fn tetrahedrons(self)
        -> Tessellate<Self, Quad<T>, Triangle<T>, (), Vec<Triangle<T>>,
                     fn(Quad<T>, ()) -> Vec<Triangle<T>>>
    {
        Tessellate::new(self, (), into_tetrahedrons)
    }
}

fn into_points<P, T>(primitive: P, _: ()) -> Vec<T>
    where P: Primitive<Point = T>,
          T: Clone
{
    let mut points = vec![];
    primitive.into_points(|point| points.push(point));
    points
}

fn into_triangles<P, T>(polygon: P, _: ()) -> Vec<Triangle<T>>
    where P: Polygonal<Point = T>,
          T: Clone
{
    let mut triangles = vec![];
    polygon.into_triangles(|triangle| triangles.push(triangle));
    triangles
}

fn into_subdivisions<P, T>(polygon: P, n: usize) -> Vec<P>
    where P: IntoSubdivisions<Point = T>,
          T: Clone + Interpolate
{
    let mut polygons = vec![];
    polygon.into_subdivisions(n, |polygon| polygons.push(polygon));
    polygons
}

fn into_tetrahedrons<T>(quad: Quad<T>, _: ()) -> Vec<Triangle<T>>
    where T: Clone + Interpolate
{
    let mut triangles = vec![];
    quad.into_tetrahedrons(|triangle| triangles.push(triangle));
    triangles
}

fn n_map_polygon<T, P, F>(n: usize, polygon: P, f: F) -> Vec<P>
    where P: Polygonal<Point = T>,
          T: Clone,
          F: Fn(P) -> Vec<P>
{
    let mut queued = vec![];
    let mut transformed = vec![polygon];
    for _ in 0..n {
        queued.extend(transformed.drain(..));
        while let Some(polygon) = queued.pop() {
            transformed.extend(f(polygon));
        }
    }
    transformed
}
