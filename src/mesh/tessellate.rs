use std::collections::VecDeque;
use std::iter::IntoIterator;
use std::marker::PhantomData;

use math::{self, FScalar};
use super::primitive::{Polygon, Polygonal, Triangle, Quad};

pub struct Tessellate<I, P, Q, D, R, F>
    where D: Copy,
          F: Fn(P, D) -> R,
          R: IntoIterator<Item = Q>
{
    source: I,
    sink: VecDeque<Q>,
    parameter: D,
    f: F,
    phantom: PhantomData<(P, R)>,
}

impl<I, P, Q, D, R, F> Tessellate<I, P, Q, D, R, F>
    where D: Copy,
          F: Fn(P, D) -> R,
          R: IntoIterator<Item = Q>
{
    pub(super) fn new(source: I, parameter: D, f: F) -> Self {
        Tessellate {
            source: source,
            sink: VecDeque::new(),
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
            if let Some(polygon) = self.sink.pop_front() {
                return Some(polygon);
            }
            if let Some(polygon) = self.source.next() {
                self.sink.extend((self.f)(polygon, self.parameter));
            }
            else {
                return None;
            }
        }
    }
}

pub trait Interpolate: math::Interpolate<FScalar> {}

impl<T> Interpolate for T where T: math::Interpolate<FScalar> {}

pub trait PolygonalExt<T>: Polygonal<T>
    where T: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, f: F) where F: FnMut(Polygon<T>);
}

pub trait QuadExt<T>
    where T: Clone + Interpolate
{
    fn into_tetrahedrons<F>(self, f: F) where F: FnMut(Triangle<T>);
}

impl<T> PolygonalExt<T> for Triangle<T>
    where T: Clone + Interpolate
{
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn into_subdivisions<F>(self, n: usize, mut f: F)
        where F: FnMut(Polygon<T>)
    {
        for triangle in n_map_polygon(n, self, |triangle| {
            let Triangle { a, b, c } = triangle;
            let ac = a.midpoint(&c);
            vec![Triangle::new(b.clone(), ac.clone(), a),
                 Triangle::new(c, ac, b)]
        }) {
            f(Polygon::Triangle(triangle));
        }
    }
}

impl<T> PolygonalExt<T> for Quad<T>
    where T: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, mut f: F)
        where F: FnMut(Polygon<T>)
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
            f(Polygon::Quad(quad));
        }
    }
}

impl<T> QuadExt<T> for Quad<T>
    where T: Clone + Interpolate
{
    fn into_tetrahedrons<F>(self, mut f: F)
        where F: FnMut(Triangle<T>)
    {
        let Quad { a, b, c, d } = self;
        let ac = a.midpoint(&c); // Diagonal.
        f(Triangle::new(a.clone(), b.clone(), ac.clone()));
        f(Triangle::new(b, c.clone(), ac.clone()));
        f(Triangle::new(c, d.clone(), ac.clone()));
        f(Triangle::new(d, a, ac));
    }
}

impl<T> PolygonalExt<T> for Polygon<T>
    where T: Clone + Interpolate
{
    fn into_subdivisions<F>(self, n: usize, f: F)
        where F: FnMut(Polygon<T>)
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_subdivisions(n, f),
            Polygon::Quad(quad) => quad.into_subdivisions(n, f),
        }
    }
}

pub trait TessellatePolygon<P, Q>: Sized {
    fn subdivide(self, n: usize) -> Tessellate<Self, P, Q, usize, Vec<Q>, fn(P, usize) -> Vec<Q>>;
}

impl<I, P, T> TessellatePolygon<P, Polygon<T>> for I
    where I: Iterator<Item = P>,
          T: Clone + Interpolate,
          P: PolygonalExt<T>
{
    fn subdivide(self, n: usize)
        -> Tessellate<Self, P, Polygon<T>, usize, Vec<Polygon<T>>, fn(P, usize) -> Vec<Polygon<T>>> {
        Tessellate::new(self, n, into_subdivisions)
    }
}

pub trait TessellateQuad<T>: Sized {
    fn tetrahedrons(self)
        -> Tessellate<Self, Quad<T>, Triangle<T>, (), Vec<Triangle<T>>,
                     fn(Quad<T>, ()) -> Vec<Triangle<T>>>;
}

impl<I, T> TessellateQuad<T> for I
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

fn into_subdivisions<P, T>(polygon: P, n: usize) -> Vec<Polygon<T>>
    where P: PolygonalExt<T>,
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
    where P: Polygonal<T>,
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
