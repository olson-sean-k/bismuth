use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;

use math;

pub trait Primitive<T>: Sized {
    fn into_points<F>(self, f: F) where F: FnMut(T);
    fn into_lines<F>(self, f: F) where F: FnMut(Line<T>);
}

pub trait Polygonal<T>: Primitive<T> {
    fn into_triangles<F>(self, f: F) where F: FnMut(Triangle<T>);
}

pub trait MapPrimitiveInto<T, U> {
    type Output;

    fn map_points_into<F>(self, f: F) -> Self::Output where F: FnMut(T) -> U;
}

pub trait MapPrimitive<T, U>: Sized {
    type Output;

    fn map_points<F>(self, f: F) -> Map<Self, T, U, F> where F: FnMut(T) -> U;
}

pub trait RotatePrimitive {
    fn rotate(&mut self, n: isize);
}

pub trait DecomposePrimitive<T>: Sized {
    fn points(self) -> PointIter<Self, T>;
}

pub trait DecomposePolygon<T>: Sized {
    fn triangulate(self) -> TriangleIter<Self, T>;
}

impl<I, T, U, P, Q> MapPrimitive<T, U> for I
    where I: Iterator<Item = P>,
          P: MapPrimitiveInto<T, U, Output = Q>
{
    type Output = Q;

    fn map_points<F>(self, f: F) -> Map<Self, T, U, F>
        where F: FnMut(T) -> U
    {
        Map::new(self, f)
    }
}

pub struct Map<I, T, U, F> {
    primitives: I,
    f: F,
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
}

impl<I, T, U, F> Map<I, T, U, F> {
    fn new(primitives: I, f: F) -> Self {
        Map {
            primitives: primitives,
            f: f,
            phantom_t: PhantomData,
            phantom_u: PhantomData,
        }
    }
}

impl<I, T, U, F, P, Q> Iterator for Map<I, T, U, F>
    where I: Iterator<Item = P>,
          F: FnMut(T) -> U,
          P: MapPrimitiveInto<T, U, Output = Q>
{
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        self.primitives.next().map(|primitive| primitive.map_points_into(|point| (self.f)(point)))
    }
}

impl<I, T, P> DecomposePrimitive<T> for I
    where I: Iterator<Item = P>,
          P: Primitive<T>
{
    fn points(self) -> PointIter<Self, T> {
        PointIter::new(self)
    }
}

pub struct PointIter<I, T> {
    primitives: I,
    points: VecDeque<T>,
}

impl<I, T> PointIter<I, T> {
    fn new(primitives: I) -> Self {
        PointIter {
            primitives: primitives,
            points: VecDeque::new(),
        }
    }
}

impl<I, T, P> Iterator for PointIter<I, T>
    where I: Iterator<Item = P>,
          P: Primitive<T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(point) = self.points.pop_front() {
                return Some(point);
            }
            if let Some(primitive) = self.primitives.next() {
                primitive.into_points(|point| self.points.push_back(point))
            }
            else {
                return None;
            }
        }
    }
}

impl<I, T, P> DecomposePolygon<T> for I
    where I: Iterator<Item = P>,
          P: Polygonal<T>
{
    fn triangulate(self) -> TriangleIter<Self, T> {
        TriangleIter::new(self)
    }
}

pub struct TriangleIter<I, T> {
    polygons: I,
    triangles: VecDeque<Triangle<T>>,
}

impl<I, T> TriangleIter<I, T> {
    fn new(polygons: I) -> Self {
        TriangleIter {
            polygons: polygons,
            triangles: VecDeque::new(),
        }
    }
}

impl<I, T, P> Iterator for TriangleIter<I, T>
    where I: Iterator<Item = P>,
          P: Polygonal<T>
{
    type Item = Triangle<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(triangle) = self.triangles.pop_front() {
                return Some(triangle);
            }
            if let Some(polygon) = self.polygons.next() {
                polygon.into_triangles(|triangle| self.triangles.push_back(triangle))
            }
            else {
                return None;
            }
        }
    }
}

pub struct Line<T> {
    pub a: T,
    pub b: T,
}

impl<T> Line<T> {
    pub fn new(a: T, b: T) -> Self {
        Line { a: a, b: b }
    }

    pub fn converged(value: T) -> Self
        where T: Clone
    {
        Line::new(value.clone(), value)
    }
}

impl<T, U> MapPrimitiveInto<T, U> for Line<T> {
    type Output = Line<U>;

    fn map_points_into<F>(self, mut f: F) -> Self::Output
        where F: FnMut(T) -> U
    {
        let Line { a, b } = self;
        Line::new(f(a), f(b))
    }
}

impl<T> Primitive<T> for Line<T>
    where T: Clone
{
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

impl<T> RotatePrimitive for Line<T>
    where T: Clone
{
    fn rotate(&mut self, n: isize) {
        if n % 2 != 0 {
            mem::swap(&mut self.a, &mut self.b);
        }
    }
}

pub struct Triangle<T> {
    pub a: T,
    pub b: T,
    pub c: T,
}

impl<T> Triangle<T> {
    pub fn new(a: T, b: T, c: T) -> Self {
        Triangle { a: a, b: b, c: c }
    }

    pub fn converged(value: T) -> Self
        where T: Clone
    {
        Triangle::new(value.clone(), value.clone(), value)
    }
}

impl<T, U> MapPrimitiveInto<T, U> for Triangle<T> {
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

impl<T> RotatePrimitive for Triangle<T>
    where T: Clone
{
    fn rotate(&mut self, n: isize) {
        let n = math::umod(n, 3);
        if n == 1 {
            mem::swap(&mut self.a, &mut self.b);
            mem::swap(&mut self.b, &mut self.c);
        }
        else if n == 2 {
            mem::swap(&mut self.c, &mut self.b);
            mem::swap(&mut self.b, &mut self.a);
        }
    }
}

pub struct Quad<T> {
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
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

    pub fn converged(value: T) -> Self
        where T: Clone
    {
        Quad::new(value.clone(), value.clone(), value.clone(), value)
    }
}

impl<T, U> MapPrimitiveInto<T, U> for Quad<T> {
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
        f(Triangle::new(a.clone(), b, c.clone()));
        f(Triangle::new(c, d, a));
    }
}

impl<T> RotatePrimitive for Quad<T>
    where T: Clone
{
    fn rotate(&mut self, n: isize) {
        let n = math::umod(n, 4);
        if n == 1 {
            mem::swap(&mut self.a, &mut self.b);
            mem::swap(&mut self.b, &mut self.c);
            mem::swap(&mut self.c, &mut self.d);
        }
        else if n == 2 {
            mem::swap(&mut self.a, &mut self.c);
            mem::swap(&mut self.b, &mut self.d);
        }
        else if n == 3 {
            mem::swap(&mut self.d, &mut self.c);
            mem::swap(&mut self.c, &mut self.b);
            mem::swap(&mut self.b, &mut self.a);
        }
    }
}

pub enum Polygon<T> {
    Triangle(Triangle<T>),
    Quad(Quad<T>),
}

impl<T, U> MapPrimitiveInto<T, U> for Polygon<T> {
    type Output = Polygon<U>;

    fn map_points_into<F>(self, f: F) -> Self::Output
        where F: FnMut(T) -> U
    {
        match self {
            Polygon::Triangle(triangle) => Polygon::Triangle(triangle.map_points_into(f)),
            Polygon::Quad(quad) => Polygon::Quad(quad.map_points_into(f)),
        }
    }
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

impl<T> RotatePrimitive for Polygon<T>
    where T: Clone
{
    fn rotate(&mut self, n: isize) {
        match *self {
            Polygon::Triangle(ref mut triangle) => triangle.rotate(n),
            Polygon::Quad(ref mut quad) => quad.rotate(n),
        }
    }
}
