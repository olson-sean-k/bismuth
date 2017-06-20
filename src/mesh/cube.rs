use nalgebra::{Point2, Point3, Scalar};
use std::ops::Range;

use super::generate::{ConjointPointGenerator, IndexPolygonGenerator, PolygonGenerator,
                      TexturePolygonGenerator};
use super::primitive::{MapPrimitiveInto, Quad};

pub trait Unit: Scalar {
    fn unit_radius() -> (Self, Self);
    fn unit_width() -> (Self, Self);
}

macro_rules! unit {
    (integer => $($t:ty),*) => {$(
        impl Unit for $t {
            fn unit_radius() -> (Self, Self) {
                use num::{One, Zero};
                (Self::zero(), Self::one() + Self::one())
            }

            fn unit_width() -> (Self, Self) {
                use num::{One, Zero};
                (Self::zero(), Self::one())
            }
        }
    )*};
    (real => $($t:ty),*) => {$(
        impl Unit for $t {
            fn unit_radius() -> (Self, Self) {
                use num::One;
                (-Self::one(), Self::one())
            }

            fn unit_width() -> (Self, Self) {
                use num::One;
                let half = Self::one() / (Self::one() + Self::one());
                (-half, half)
            }
        }
    )*};
}

unit!(integer => i8, i16, i32, i64, u8, u16, u32, u64);
unit!(real => f32, f64);

#[derive(Clone, Copy)]
pub enum FacePlane {
    XY,
    NXY,
    ZY,
    NZY,
    XZ,
    XNZ,
}

#[derive(Clone)]
pub struct Cube<T>
    where T: Unit
{
    faces: Range<usize>,
    lower: T,
    upper: T,
}

impl<T> Cube<T>
    where T: Unit
{
    fn new(lower: T, upper: T) -> Self {
        Cube {
            faces: 0..6,
            lower: lower,
            upper: upper,
        }
    }

    pub fn with_unit_radius() -> Self {
        let (lower, upper) = T::unit_radius();
        Cube::new(lower, upper)
    }

    pub fn with_unit_width() -> Self {
        let (lower, upper) = T::unit_width();
        Cube::new(lower, upper)
    }

    pub fn plane_polygons(&self) -> PlanePolygonIter {
        PlanePolygonIter::new(0..self.polygon_count())
    }

    fn point(&self, index: usize) -> Point3<T> {
        point(index, self.lower, self.upper)
    }

    fn face(&self, index: usize) -> Quad<Point3<T>> {
        index_face(index).map_points_into(|index| self.point(index))
    }
}

impl<T> Iterator for Cube<T>
    where T: Unit
{
    type Item = Quad<Point3<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|index| self.face(index))
    }
}

pub struct PlanePolygonIter {
    polygons: Range<usize>,
}

impl PlanePolygonIter {
    fn new(polygons: Range<usize>) -> Self {
        PlanePolygonIter {
            polygons: polygons,
        }
    }
}

impl Iterator for PlanePolygonIter {
    type Item = Quad<FacePlane>;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| plane_face(index))
    }
}

impl<T> ConjointPointGenerator<Point3<T>> for Cube<T>
    where T: Unit
{
    fn conjoint_point(&self, index: usize) -> Point3<T> {
        self.point(index)
    }

    fn conjoint_point_count(&self) -> usize {
        8
    }
}

impl<T> PolygonGenerator for Cube<T>
    where T: Unit
{
    fn polygon_count(&self) -> usize {
        6
    }
}

impl<T> IndexPolygonGenerator<Quad<usize>> for Cube<T>
    where T: Unit
{
    fn index_polygon(&self, index: usize) -> Quad<usize> {
        index_face(index)
    }
}

impl<T> TexturePolygonGenerator<Quad<Point2<f32>>> for Cube<T>
    where T: Unit
{
    fn texture_polygon(&self, index: usize) -> Quad<Point2<f32>> {
        texture_face(index)
    }
}

fn point<T>(index: usize, lower: T, upper: T) -> Point3<T>
    where T: Unit
{
    let x = if index & 0b100 == 0b100 { upper } else { lower };
    let y = if index & 0b010 == 0b010 { upper } else { lower };
    let z = if index & 0b001 == 0b001 { upper } else { lower };
    Point3::new(x, y, z)
}

fn index_face(index: usize) -> Quad<usize> {
    match index {
        0 => Quad::new(5, 7, 3, 1), // front
        1 => Quad::new(6, 7, 5, 4), // right
        2 => Quad::new(3, 7, 6, 2), // top
        3 => Quad::new(0, 1, 3, 2), // left
        4 => Quad::new(4, 5, 1, 0), // bottom
        5 => Quad::new(0, 2, 6, 4), // back
        _ => panic!(),
    }
}

fn texture_face(index: usize) -> Quad<Point2<f32>> {
    let uu = Point2::new(1.0, 1.0);
    let ul = Point2::new(1.0, 0.0);
    let ll = Point2::new(0.0, 0.0);
    let lu = Point2::new(0.0, 1.0);
    match index {
        0 => Quad::new(uu, ul, ll, lu), // front
        1 => Quad::new(ul, ll, lu, uu), // right
        2 => Quad::new(lu, uu, ul, ll), // top
        3 => Quad::new(lu, uu, ul, ll), // left
        4 => Quad::new(uu, ul, ll, lu), // bottom
        5 => Quad::new(uu, ul, ll, lu), // back
        _ => panic!(),
    }
}

fn plane_face(index: usize) -> Quad<FacePlane> {
    match index {
        0 => Quad::converged(FacePlane::XY),  // front
        1 => Quad::converged(FacePlane::NZY), // right
        2 => Quad::converged(FacePlane::XNZ), // top
        3 => Quad::converged(FacePlane::ZY),  // left
        4 => Quad::converged(FacePlane::XZ),  // bottom
        5 => Quad::converged(FacePlane::NXY), // back
        _ => panic!(),
    }
}
