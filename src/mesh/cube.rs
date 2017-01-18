use nalgebra::Point3;
use num::{Float, Unsigned};
use std::ops::Range;

use super::generate::{Conjoint, Indexed};
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

pub struct FCube<T>
    where T: Float
{
    faces: Range<usize>,
    low: T,
    high: T,
}

impl<T> FCube<T>
    where T: Float
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
    where T: Float
{
    type Point = Point3<T>;

    fn point(&self, index: usize) -> Self::Point {
        point(index, self.low, self.high)
    }
}

impl<T> Iterator for FCube<T>
    where T: Float
{
    type Item = Quad<<Self as Cube>::Point>;

    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|index| self.face(index))
    }
}

pub struct UCube<T>
    where T: Copy + Unsigned
{
    faces: Range<usize>,
    low: T,
    high: T,
}

impl<T> UCube<T>
    where T: Copy + Unsigned
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
    where T: Copy + Unsigned
{
    type Point = Point3<T>;

    fn point(&self, index: usize) -> Self::Point {
        point(index, self.low, self.high)
    }
}

impl<T> Iterator for UCube<T>
    where T: Copy + Unsigned
{
    type Item = Quad<<Self as Cube>::Point>;

    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|index| self.face(index))
    }
}

fn point<T>(index: usize, low: T, high: T) -> Point3<T>
    where T: Copy
{
    let x = if index & 0b100 == 0b100 { high } else { low };
    let y = if index & 0b010 == 0b010 { high } else { low };
    let z = if index & 0b001 == 0b001 { high } else { low };
    Point3::new(x, y, z)
}
