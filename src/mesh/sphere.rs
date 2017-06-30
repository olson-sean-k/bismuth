use nalgebra::{Point3, Scalar};
use num::Float;
use num::traits::FloatConst;
use std::cmp;
use std::marker::PhantomData;

use super::generate::{ConjointPointGenerator, IndexPolygonGenerator, PolygonGenerator};
use super::primitive::{Polygon, Triangle, Quad};

#[derive(Clone)]
pub struct UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    nu: usize, // Meridians.
    nv: usize, // Parallels.
    phantom: PhantomData<T>,
}

impl<T> UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    pub fn with_unit_radius(nu: usize, nv: usize) -> Self {
        let nu = cmp::max(3, nu);
        let nv = cmp::max(2, nv);
        UVSphere {
            nu: nu,
            nv: nv,
            phantom: PhantomData,
        }
    }

    fn point(&self, u: usize, v: usize) -> Point3<T> {
        let u = (T::from(u).unwrap() / T::from(self.nu).unwrap()) * T::PI() * (T::one() + T::one());
        let v = (T::from(v).unwrap() / T::from(self.nv).unwrap()) * T::PI();
        Point3::new(u.cos() * v.sin(), u.sin() * v.sin(), v.cos())
    }

    fn index_point(&self, u: usize, v: usize) -> usize {
        if v == 0 {
            0
        }
        else if v == self.nv {
            ((self.nv - 1) * self.nu) + 1
        }
        else {
            ((v - 1) * self.nu) + (u % self.nu) + 1
        }
    }

    fn map_polygon_index(&self, index: usize) -> (usize, usize) {
        (index % self.nu, index / self.nu)
    }
}

impl<T> ConjointPointGenerator for UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    type Output = Point3<T>;

    fn conjoint_point(&self, index: usize) -> Self::Output {
        if index == 0 {
            self.point(0, 0)
        }
        else if index == self.conjoint_point_count() - 1 {
            self.point(0, self.nv)
        }
        else {
            let index = index - 1;
            self.point(index % self.nu, (index / self.nv) + 1)
        }
    }

    fn conjoint_point_count(&self) -> usize {
        (self.nv - 1) * self.nu + 2
    }
}

impl<T> PolygonGenerator for UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    type Output = Polygon<Point3<T>>;

    fn polygon(&self, index: usize) -> Self::Output {
        let (u, v) = self.map_polygon_index(index);

        // Generate the points at the requested meridian and parallel. The
        // upper and lower bounds of (u, v) are always used, so generate them
        // in advance (`low` and `high`). Emit triangles at the poles,
        // otherwise quads.
        let low = self.point(u, v);
        let high = self.point(u + 1, v + 1);
        if v == 0 {
            Polygon::Triangle(Triangle::new(low, self.point(u, v + 1), high))
        }
        else if v == self.nv - 1 {
            Polygon::Triangle(Triangle::new(high, self.point(u + 1, v), low))
        }
        else {
            Polygon::Quad(Quad::new(
                low,
                self.point(u, v + 1),
                high,
                self.point(u + 1, v),
            ))
        }
    }

    fn polygon_count(&self) -> usize {
        self.nu * self.nv
    }
}

impl<T> IndexPolygonGenerator for UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    type Output = Polygon<usize>;

    fn index_polygon(&self, index: usize) -> <Self as IndexPolygonGenerator>::Output {
        let (u, v) = self.map_polygon_index(index);

        let low = self.index_point(u, v);
        let high = self.index_point(u + 1, v + 1);
        if v == 0 {
            Polygon::Triangle(Triangle::new(low, self.index_point(u, v + 1), high))
        }
        else if v == self.nv - 1 {
            Polygon::Triangle(Triangle::new(high, self.index_point(u + 1, v), low))
        }
        else {
            Polygon::Quad(Quad::new(
                low,
                self.index_point(u, v + 1),
                high,
                self.index_point(u + 1, v),
            ))
        }
    }
}
