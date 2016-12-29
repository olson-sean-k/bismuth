use std::collections::VecDeque;
use std::marker::PhantomData;

pub trait Primitive<T>: Sized {
    fn into_points<F>(self, f: F) where F: FnMut(T);
    fn into_lines<F>(self, f: F) where F: FnMut(Line<T>);
}

pub trait Polygonal<T>: Primitive<T> {
    fn into_triangles<F>(self, f: F) where F: FnMut(Triangle<T>);
}

pub trait MapInto<T, U> {
    type Output;

    fn map_points_into<F>(self, f: F) -> Self::Output where F: FnMut(T) -> U;
}

pub trait Map<T, U>: Sized {
    type Output;

    fn map_points<F>(self, f: F) -> MapPoint<Self, T, U, F> where F: FnMut(T) -> U;
}

pub trait Triangulate<T>: Sized {
    fn triangulate(self) -> TriangulatePolygon<Self, T>;
}

impl<I, T, U, P, Q> Map<T, U> for I
    where I: Iterator<Item = P>,
          P: MapInto<T, U, Output = Q>
{
    type Output = Q;

    fn map_points<F>(self, f: F) -> MapPoint<Self, T, U, F>
        where F: FnMut(T) -> U
    {
        MapPoint::new(self, f)
    }
}

pub struct MapPoint<I, T, U, F> {
    primitives: I,
    f: F,
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
}

impl<I, T, U, F> MapPoint<I, T, U, F> {
    fn new(primitives: I, f: F) -> Self {
        MapPoint {
            primitives: primitives,
            f: f,
            phantom_t: PhantomData,
            phantom_u: PhantomData,
        }
    }
}

impl<I, T, U, F, P, Q> Iterator for MapPoint<I, T, U, F>
    where I: Iterator<Item = P>,
          F: FnMut(T) -> U,
          P: MapInto<T, U, Output = Q>
{
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        self.primitives.next().map(|primitive| primitive.map_points_into(|point| (self.f)(point)))
    }
}

impl<I, T, P> Triangulate<T> for I
    where I: Iterator<Item = P>,
          P: Polygonal<T>
{
    fn triangulate(self) -> TriangulatePolygon<Self, T> {
        TriangulatePolygon::new(self)
    }
}

pub struct TriangulatePolygon<I, T> {
    primitives: I,
    triangles: VecDeque<Triangle<T>>,
    phantom_t: PhantomData<T>,
}

impl<I, T> TriangulatePolygon<I, T> {
    fn new(primitives: I) -> Self {
        TriangulatePolygon {
            primitives: primitives,
            triangles: VecDeque::new(),
            phantom_t: PhantomData,
        }
    }
}

impl<I, T, P> Iterator for TriangulatePolygon<I, T>
    where I: Iterator<Item = P>,
          P: Polygonal<T>
{
    type Item = Triangle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(triangle) = self.triangles.pop_front() {
                return Some(triangle);
            }
            if let Some(primitive) = self.primitives.next() {
                primitive.into_triangles(|triangle| self.triangles.push_back(triangle))
            }
            else {
                return None;
            }
        }
    }
}

pub struct Line<T> {
    a: T,
    b: T,
}

impl<T> Line<T> {
    pub fn new(a: T, b: T) -> Self {
        Line { a: a, b: b }
    }
}

impl<T, U> MapInto<T, U> for Line<T> {
    type Output = Line<U>;

    fn map_points_into<F>(self, mut f: F) -> Self::Output
        where F: FnMut(T) -> U
    {
        let Line { a, b } = self;
        Line::new(f(a), f(b))
    }
}

impl<T> Primitive<T> for Line<T> {
    fn into_points<F>(self, mut f: F)
        where F: FnMut(T)
    {
        let Line { a, b } = self;
        f(a);
        f(b);
    }

    fn into_lines<F>(self, mut f: F)
        where F: FnMut(Line<T>)
    {
        f(self);
    }
}

pub struct Triangle<T> {
    a: T,
    b: T,
    c: T,
}

impl<T> Triangle<T> {
    pub fn new(a: T, b: T, c: T) -> Self {
        Triangle { a: a, b: b, c: c }
    }
}

impl<T, U> MapInto<T, U> for Triangle<T> {
    type Output = Triangle<U>;

    fn map_points_into<F>(self, mut f: F) -> Self::Output
        where F: FnMut(T) -> U
    {
        let Triangle { a, b, c } = self;
        Triangle::new(f(a), f(b), f(c))
    }
}

impl<T> Primitive<T> for Triangle<T>
    where T: Clone
{
    fn into_points<F>(self, mut f: F)
        where F: FnMut(T)
    {
        let Triangle { a, b, c } = self;
        f(a);
        f(b);
        f(c);
    }

    fn into_lines<F>(self, mut f: F)
        where F: FnMut(Line<T>)
    {
        let Triangle { a, b, c } = self;
        f(Line::new(a.clone(), b.clone()));
        f(Line::new(b, c.clone()));
        f(Line::new(c, a));
    }
}

impl<T> Polygonal<T> for Triangle<T>
    where T: Clone
{
    fn into_triangles<F>(self, mut f: F)
        where F: FnMut(Triangle<T>)
    {
        f(self);
    }
}

pub struct Quad<T> {
    a: T,
    b: T,
    c: T,
    d: T,
}

impl<T> Quad<T> {
    pub fn new(a: T, b: T, c: T, d: T) -> Self {
        Quad {
            a: a,
            b: b,
            c: c,
            d: d,
        }
    }
}

impl<T, U> MapInto<T, U> for Quad<T> {
    type Output = Quad<U>;

    fn map_points_into<F>(self, mut f: F) -> Self::Output
        where F: FnMut(T) -> U
    {
        let Quad { a, b, c, d } = self;
        Quad::new(f(a), f(b), f(c), f(d))
    }
}

impl<T> Primitive<T> for Quad<T>
    where T: Clone
{
    fn into_points<F>(self, mut f: F)
        where F: FnMut(T)
    {
        let Quad { a, b, c, d } = self;
        f(a);
        f(b);
        f(c);
        f(d);
    }

    fn into_lines<F>(self, mut f: F)
        where F: FnMut(Line<T>)
    {
        let Quad { a, b, c, d } = self;
        f(Line::new(a.clone(), b.clone()));
        f(Line::new(b, c.clone()));
        f(Line::new(c, d.clone()));
        f(Line::new(d, a));
    }
}

impl<T> Polygonal<T> for Quad<T>
    where T: Clone
{
    fn into_triangles<F>(self, mut f: F)
        where F: FnMut(Triangle<T>)
    {
        let Quad { a, b, c, d } = self;
        f(Triangle::new(a, b.clone(), c.clone()));
        f(Triangle::new(c, b, d));
    }
}

pub enum Polygon<T> {
    Triangle(Triangle<T>),
    Quad(Quad<T>),
}

impl<T> Primitive<T> for Polygon<T>
    where T: Clone
{
    fn into_points<F>(self, f: F)
        where F: FnMut(T)
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_points(f),
            Polygon::Quad(quad) => quad.into_points(f),
        }
    }

    fn into_lines<F>(self, f: F)
        where F: FnMut(Line<T>)
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_lines(f),
            Polygon::Quad(quad) => quad.into_lines(f),
        }
    }
}

impl<T> Polygonal<T> for Polygon<T>
    where T: Clone
{
    fn into_triangles<F>(self, f: F)
        where F: FnMut(Triangle<T>)
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_triangles(f),
            Polygon::Quad(quad) => quad.into_triangles(f),
        }
    }
}
