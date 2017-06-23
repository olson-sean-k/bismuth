use nalgebra::{Point2, Point3, Scalar};

use super::generate::{ConjointPointGenerator, IndexPolygonGenerator, Generate, PolygonGenerator,
                      TexturePolygonGenerator};
use super::primitive::{MapPrimitive, Quad};

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
    lower: T,
    upper: T,
}

impl<T> Cube<T>
    where T: Unit
{
    fn new(lower: T, upper: T) -> Self {
        Cube {
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

    pub fn polygons<'a>(&'a self)
        -> Generate<'a, Self, Quad<Point3<T>>, fn(&'a Self, usize) -> Quad<Point3<T>>>
    {
        Generate::new(self, 0..self.polygon_count(), map_polygon)
    }

    pub fn plane_polygons<'a>(&'a self)
        -> Generate<'a, Self, Quad<FacePlane>, fn(&'a Self, usize) -> Quad<FacePlane>>
    {
        Generate::new(self, 0..self.polygon_count(), map_plane_polygon)
    }

    fn point(&self, index: usize) -> Point3<T> {
        let x = if index & 0b100 == 0b100 { self.upper } else { self.lower };
        let y = if index & 0b010 == 0b010 { self.upper } else { self.lower };
        let z = if index & 0b001 == 0b001 { self.upper } else { self.lower };
        Point3::new(x, y, z)
    }

    fn face(&self, index: usize) -> Quad<Point3<T>> {
        index_face(index).map_primitive(|index| self.point(index))
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

fn map_polygon<T>(source: &Cube<T>, index: usize) -> Quad<Point3<T>>
    where T: Unit
{
    source.face(index)
}

fn map_plane_polygon<T>(_: &Cube<T>, index: usize) -> Quad<FacePlane>
    where T: Unit
{
    plane_face(index)
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
