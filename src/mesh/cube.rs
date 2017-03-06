use nalgebra::{Point2, Point3, Scalar};
use num::{Float, Unsigned};
use std::ops::Range;

use super::generate::{Conjoint, Indexed, Textured};
use super::primitive::{MapPrimitiveInto, Quad};

pub trait Cube: Iterator + Sized {
    type Point;

    fn point(&self, index: usize) -> Self::Point;

    fn face(&self, index: usize) -> Quad<Self::Point> {
        self.indexed_face(index).map_points_into(|index| self.point(index))
    }

    fn indexed_face(&self, index: usize) -> Quad<usize> {
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

    fn textured_face(&self, index: usize) -> Quad<Point2<f32>> {
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
}

impl<T, C> Conjoint<T> for C
    where C: Cube<Point = T>
{
    fn conjoint_point(&self, index: usize) -> T {
        self.point(index)
    }

    fn conjoint_point_count(&self) -> usize {
        8
    }
}

impl<C> Indexed<Quad<usize>> for C
    where C: Cube
{
    fn indexed_polygon(&self, index: usize) -> Quad<usize> {
        self.indexed_face(index)
    }

    fn indexed_polygon_count(&self) -> usize {
        6
    }
}

impl<C> Textured<Quad<Point2<f32>>> for C
    where C: Cube
{
    fn textured_polygon(&self, index: usize) -> Quad<Point2<f32>> {
        self.textured_face(index)
    }

    fn textured_polygon_count(&self) -> usize {
        6
    }
}

#[derive(Clone)]
pub struct RCube<T>
    where T: Float + Scalar
{
    faces: Range<usize>,
    low: T,
    high: T,
}

impl<T> RCube<T>
    where T: Float + Scalar
{
    fn new(low: T, high: T) -> Self {
        RCube {
            faces: 0..6,
            low: low,
            high: high,
        }
    }

    pub fn with_unit_radius() -> Self {
        RCube::new(-T::one(), T::one())
    }

    pub fn with_unit_width() -> Self {
        let half = T::one() / (T::one() + T::one());
        RCube::new(-half, half)
    }
}

impl<T> Cube for RCube<T>
    where T: Float + Scalar
{
    type Point = Point3<T>;

    fn point(&self, index: usize) -> Self::Point {
        point(index, self.low, self.high)
    }
}

impl<T> Iterator for RCube<T>
    where T: Float + Scalar
{
    type Item = Quad<<Self as Cube>::Point>;

    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|index| self.face(index))
    }
}

#[derive(Clone)]
pub struct NCube<T>
    where T: Unsigned + Scalar
{
    faces: Range<usize>,
    low: T,
    high: T,
}

impl<T> NCube<T>
    where T: Unsigned + Scalar
{
    fn new(low: T, high: T) -> Self {
        NCube {
            faces: 0..6,
            low: low,
            high: high,
        }
    }

    pub fn with_unit_radius() -> Self {
        NCube::new(T::zero(), T::one() + T::one())
    }

    pub fn with_unit_width() -> Self {
        NCube::new(T::zero(), T::one())
    }
}

impl<T> Cube for NCube<T>
    where T: Unsigned + Scalar
{
    type Point = Point3<T>;

    fn point(&self, index: usize) -> Self::Point {
        point(index, self.low, self.high)
    }
}

impl<T> Iterator for NCube<T>
    where T: Unsigned + Scalar
{
    type Item = Quad<<Self as Cube>::Point>;

    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|index| self.face(index))
    }
}

fn point<T>(index: usize, low: T, high: T) -> Point3<T>
    where T: Scalar
{
    let x = if index & 0b100 == 0b100 { high } else { low };
    let y = if index & 0b010 == 0b010 { high } else { low };
    let z = if index & 0b001 == 0b001 { high } else { low };
    Point3::new(x, y, z)
}
