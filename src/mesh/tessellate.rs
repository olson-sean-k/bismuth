use nalgebra::Point3;
use num::Num;
use std::collections::VecDeque;

use super::primitive::{Polygon, Polygonal, Triangle, Quad};

pub trait Interpolate {
    fn interpolate(&self, other: &Self) -> Self;
}

impl<T> Interpolate for (T, T, T)
    where T: Copy + Num
{
    fn interpolate(&self, other: &Self) -> Self {
        let two = T::one() + T::one();
        ((self.0 + other.0) / two, (self.1 + other.1) / two, (self.2 + other.2) / two)
    }
}

impl<T> Interpolate for Point3<T>
    where T: Copy + Num
{
    fn interpolate(&self, other: &Self) -> Self {
        let two = T::one() + T::one();
        Point3::new((self.x + other.x) / two, (self.y + other.y) / two, (self.z + other.z) / two)
    }
}

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
        for triangle in recursive_map(n, self, |triangle| {
            let Triangle { a, b, c } = triangle;
            let ac = a.interpolate(&c);
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
        for quad in recursive_map(n, self, |quad| {
            let Quad { a, b, c, d } = quad;
            let ab = a.interpolate(&b);
            let bc = b.interpolate(&c);
            let cd = c.interpolate(&d);
            let da = d.interpolate(&a);
            let ac = a.interpolate(&c); // Diagonal.
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
        let ac = a.interpolate(&c); // Diagonal.
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

fn recursive_map<T, P, F>(n: usize, polygon: P, f: F) -> Vec<P>
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
