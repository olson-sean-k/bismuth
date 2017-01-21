use std::collections::VecDeque;

use math::{self, FScalar};
use super::primitive::{Polygon, Polygonal, Triangle, Quad};

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
    fn into_subdivisions<F>(self, n: usize, mut f: F)
        where F: FnMut(Polygon<T>)
    {
        for triangle in n_map_polygon(n, self, |triangle| {
            let Triangle { a, b, c } = triangle;
            let ac = a.lerp(&c, 0.5);
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
            let ab = a.lerp(&b, 0.5);
            let bc = b.lerp(&c, 0.5);
            let cd = c.lerp(&d, 0.5);
            let da = d.lerp(&a, 0.5);
            let ac = a.lerp(&c, 0.5); // Diagonal.
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
        let ac = a.lerp(&c, 0.5); // Diagonal.
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

pub trait TessellatePolygon<T>: Sized {
    fn subdivide(self, n: usize) -> SubdivisionIter<Self, T>;
}

impl<I, T, P> TessellatePolygon<T> for I
    where I: Iterator<Item = P>,
          T: Clone + Interpolate,
          P: PolygonalExt<T>
{
    fn subdivide(self, n: usize) -> SubdivisionIter<Self, T> {
        SubdivisionIter::new(self, n)
    }
}

pub struct SubdivisionIter<I, T> {
    polygons: I,
    subdivisions: VecDeque<Polygon<T>>,
    n: usize,
}

impl<I, T> SubdivisionIter<I, T> {
    fn new(polygons: I, n: usize) -> Self {
        SubdivisionIter {
            polygons: polygons,
            subdivisions: VecDeque::new(),
            n: n,
        }
    }
}

impl<I, T, P> Iterator for SubdivisionIter<I, T>
    where I: Iterator<Item = P>,
          T: Clone + Interpolate,
          P: PolygonalExt<T>
{
    type Item = Polygon<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(subdivision) = self.subdivisions.pop_front() {
                return Some(subdivision);
            }
            if let Some(polygon) = self.polygons.next() {
                polygon.into_subdivisions(self.n, |polygon| self.subdivisions.push_back(polygon))
            }
            else {
                return None;
            }
        }
    }
}

pub trait TessellateQuad<T>: Sized {
    fn tetrahedrons(self) -> TetrahedronIter<Self, T>;
}

impl<I, T> TessellateQuad<T> for I
    where I: Iterator<Item = Quad<T>>
{
    fn tetrahedrons(self) -> TetrahedronIter<Self, T> {
        TetrahedronIter::new(self)
    }
}

pub struct TetrahedronIter<I, T> {
    quads: I,
    triangles: VecDeque<Triangle<T>>,
}

impl<I, T> TetrahedronIter<I, T> {
    fn new(quads: I) -> Self {
        TetrahedronIter {
            quads: quads,
            triangles: VecDeque::new(),
        }
    }
}

impl<I, T> Iterator for TetrahedronIter<I, T>
    where I: Iterator<Item = Quad<T>>,
          T: Clone + Interpolate
{
    type Item = Triangle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(triangle) = self.triangles.pop_front() {
                return Some(triangle);
            }
            if let Some(quad) = self.quads.next() {
                quad.into_tetrahedrons(|triangle| self.triangles.push_back(triangle))
            }
            else {
                return None;
            }
        }
    }
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
