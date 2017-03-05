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
            0 => Quad::new(5, 7, 3, 1),
            1 => Quad::new(6, 7, 5, 4),
            2 => Quad::new(3, 7, 6, 2),
            3 => Quad::new(0, 1, 3, 2),
            4 => Quad::new(4, 5, 1, 0),
            5 => Quad::new(0, 2, 6, 4),
            _ => panic!(),
        }
    }

    fn textured_face(&self, _: usize) -> Quad<Point2<f32>> {
        Quad::new(Point2::new(0.0, 1.0),
                  Point2::new(1.0, 1.0),
                  Point2::new(1.0, 0.0),
                  Point2::new(0.0, 0.0))
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
pub struct FCube<T>
    where T: Float + Scalar
{
    faces: Range<usize>,
    low: T,
    high: T,
}

impl<T> FCube<T>
    where T: Float + Scalar
{
    fn new(low: T, high: T) -> Self {
        FCube {
            faces: 0..6,
            low: low,
            high: high,
        }
    }

    pub fn with_unit_radius() -> Self {
        FCube::new(-T::one(), T::one())
    }

    pub fn with_unit_width() -> Self {
        let half = T::one() / (T::one() + T::one());
        FCube::new(-half, half)
    }
}

impl<T> Cube for FCube<T>
    where T: Float + Scalar
{
    type Point = Point3<T>;

    fn point(&self, index: usize) -> Self::Point {
        point(index, self.low, self.high)
    }
}

impl<T> Iterator for FCube<T>
    where T: Float + Scalar
{
    type Item = Quad<<Self as Cube>::Point>;

    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|index| self.face(index))
    }
}

#[derive(Clone)]
pub struct UCube<T>
    where T: Unsigned + Scalar
{
    faces: Range<usize>,
    low: T,
    high: T,
}

impl<T> UCube<T>
    where T: Unsigned + Scalar
{
    fn new(low: T, high: T) -> Self {
        UCube {
            faces: 0..6,
            low: low,
            high: high,
        }
    }

    pub fn with_unit_radius() -> Self {
        UCube::new(T::zero(), T::one() + T::one())
    }

    pub fn with_unit_width() -> Self {
        UCube::new(T::zero(), T::one())
    }
}

impl<T> Cube for UCube<T>
    where T: Unsigned + Scalar
{
    type Point = Point3<T>;

    fn point(&self, index: usize) -> Self::Point {
        point(index, self.low, self.high)
    }
}

impl<T> Iterator for UCube<T>
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
