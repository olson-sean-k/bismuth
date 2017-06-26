use std::convert::Into;
use std::marker::PhantomData;
use std::mem;

use math;

pub trait Primitive: Sized {
    type Point: Clone;

    fn into_points<F>(self, f: F)
    where
        F: FnMut(Self::Point);
    fn into_lines<F>(self, f: F)
    where
        F: FnMut(Line<Self::Point>);
}

pub trait Polygonal: Primitive {
    fn into_triangles<F>(self, f: F)
    where
        F: FnMut(Triangle<Self::Point>);
}

pub trait MapPrimitive<T, U>: Primitive<Point = T>
where
    T: Clone,
    U: Clone,
{
    type Output: Primitive<Point = U>;

    fn map_primitive<F>(self, f: F) -> Self::Output
    where
        F: FnMut(T) -> U;
}

pub trait MapPoints<T, U>: Sized
where
    T: Clone,
    U: Clone,
{
    type Output: Primitive<Point = U>;

    fn map_points<F>(self, f: F) -> Map<Self, T, U, F>
    where
        F: FnMut(T) -> U;
}

pub trait Rotate {
    fn rotate(&mut self, n: isize);
}

impl<I, T, U, P, Q> MapPoints<T, U> for I
where
    I: Iterator<Item = P>,
    P: MapPrimitive<T, U, Output = Q>,
    Q: Primitive<Point = U>,
    T: Clone,
    U: Clone,
{
    type Output = Q;

    fn map_points<F>(self, f: F) -> Map<Self, T, U, F>
    where
        F: FnMut(T) -> U,
    {
        Map::new(self, f)
    }
}

pub struct Map<I, T, U, F> {
    primitives: I,
    f: F,
    phantom: PhantomData<(T, U)>,
}

impl<I, T, U, F> Map<I, T, U, F> {
    fn new(primitives: I, f: F) -> Self {
        Map {
            primitives: primitives,
            f: f,
            phantom: PhantomData,
        }
    }
}

impl<I, T, U, F, P, Q> Iterator for Map<I, T, U, F>
where
    I: Iterator<Item = P>,
    F: FnMut(T) -> U,
    P: MapPrimitive<T, U, Output = Q>,
    Q: Primitive<Point = U>,
    T: Clone,
    U: Clone,
{
    type Item = Q;

    fn next(&mut self) -> Option<Self::Item> {
        self.primitives
            .next()
            .map(|primitive| primitive.map_primitive(|point| (self.f)(point)))
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
    where
        T: Clone,
    {
        Line::new(value.clone(), value)
    }
}

impl<T, U> MapPrimitive<T, U> for Line<T>
where
    T: Clone,
    U: Clone,
{
    type Output = Line<U>;

    fn map_primitive<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(T) -> U,
    {
        let Line { a, b } = self;
        Line::new(f(a), f(b))
    }
}

impl<T> Primitive for Line<T>
where
    T: Clone,
{
    type Point = T;

    fn into_points<F>(self, mut f: F)
    where
        F: FnMut(Self::Point),
    {
        let Line { a, b } = self;
        f(a);
        f(b);
    }

    fn into_lines<F>(self, mut f: F)
    where
        F: FnMut(Line<Self::Point>),
    {
        f(self);
    }
}

impl<T> Rotate for Line<T>
where
    T: Clone,
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
    where
        T: Clone,
    {
        Triangle::new(value.clone(), value.clone(), value)
    }
}

impl<T> Into<Polygon<T>> for Triangle<T> {
    fn into(self) -> Polygon<T> {
        Polygon::Triangle(self)
    }
}

impl<T, U> MapPrimitive<T, U> for Triangle<T>
where
    T: Clone,
    U: Clone,
{
    type Output = Triangle<U>;

    fn map_primitive<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(T) -> U,
    {
        let Triangle { a, b, c } = self;
        Triangle::new(f(a), f(b), f(c))
    }
}

impl<T> Primitive for Triangle<T>
where
    T: Clone,
{
    type Point = T;

    fn into_points<F>(self, mut f: F)
    where
        F: FnMut(Self::Point),
    {
        let Triangle { a, b, c } = self;
        f(a);
        f(b);
        f(c);
    }

    fn into_lines<F>(self, mut f: F)
    where
        F: FnMut(Line<Self::Point>),
    {
        let Triangle { a, b, c } = self;
        f(Line::new(a.clone(), b.clone()));
        f(Line::new(b, c.clone()));
        f(Line::new(c, a));
    }
}

impl<T> Polygonal for Triangle<T>
where
    T: Clone,
{
    fn into_triangles<F>(self, mut f: F)
    where
        F: FnMut(Triangle<Self::Point>),
    {
        f(self);
    }
}

impl<T> Rotate for Triangle<T>
where
    T: Clone,
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
    where
        T: Clone,
    {
        Quad::new(value.clone(), value.clone(), value.clone(), value)
    }
}

impl<T> Into<Polygon<T>> for Quad<T> {
    fn into(self) -> Polygon<T> {
        Polygon::Quad(self)
    }
}

impl<T, U> MapPrimitive<T, U> for Quad<T>
where
    T: Clone,
    U: Clone,
{
    type Output = Quad<U>;

    fn map_primitive<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(T) -> U,
    {
        let Quad { a, b, c, d } = self;
        Quad::new(f(a), f(b), f(c), f(d))
    }
}

impl<T> Primitive for Quad<T>
where
    T: Clone,
{
    type Point = T;

    fn into_points<F>(self, mut f: F)
    where
        F: FnMut(Self::Point),
    {
        let Quad { a, b, c, d } = self;
        f(a);
        f(b);
        f(c);
        f(d);
    }

    fn into_lines<F>(self, mut f: F)
    where
        F: FnMut(Line<Self::Point>),
    {
        let Quad { a, b, c, d } = self;
        f(Line::new(a.clone(), b.clone()));
        f(Line::new(b, c.clone()));
        f(Line::new(c, d.clone()));
        f(Line::new(d, a));
    }
}

impl<T> Polygonal for Quad<T>
where
    T: Clone,
{
    fn into_triangles<F>(self, mut f: F)
    where
        F: FnMut(Triangle<Self::Point>),
    {
        let Quad { a, b, c, d } = self;
        f(Triangle::new(a.clone(), b, c.clone()));
        f(Triangle::new(c, d, a));
    }
}

impl<T> Rotate for Quad<T>
where
    T: Clone,
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

impl<T, U> MapPrimitive<T, U> for Polygon<T>
where
    T: Clone,
    U: Clone,
{
    type Output = Polygon<U>;

    fn map_primitive<F>(self, f: F) -> Self::Output
    where
        F: FnMut(T) -> U,
    {
        match self {
            Polygon::Triangle(triangle) => Polygon::Triangle(triangle.map_primitive(f)),
            Polygon::Quad(quad) => Polygon::Quad(quad.map_primitive(f)),
        }
    }
}

impl<T> Primitive for Polygon<T>
where
    T: Clone,
{
    type Point = T;

    fn into_points<F>(self, f: F)
    where
        F: FnMut(Self::Point),
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_points(f),
            Polygon::Quad(quad) => quad.into_points(f),
        }
    }

    fn into_lines<F>(self, f: F)
    where
        F: FnMut(Line<Self::Point>),
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_lines(f),
            Polygon::Quad(quad) => quad.into_lines(f),
        }
    }
}

impl<T> Polygonal for Polygon<T>
where
    T: Clone,
{
    fn into_triangles<F>(self, f: F)
    where
        F: FnMut(Triangle<Self::Point>),
    {
        match self {
            Polygon::Triangle(triangle) => triangle.into_triangles(f),
            Polygon::Quad(quad) => quad.into_triangles(f),
        }
    }
}

impl<T> Rotate for Polygon<T>
where
    T: Clone,
{
    fn rotate(&mut self, n: isize) {
        match *self {
            Polygon::Triangle(ref mut triangle) => triangle.rotate(n),
            Polygon::Quad(ref mut quad) => quad.rotate(n),
        }
    }
}
