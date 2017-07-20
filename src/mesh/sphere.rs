use nalgebra::{Point3, Scalar};
use num::Float;
use num::traits::FloatConst;
use std::cmp;
use std::marker::PhantomData;

use super::generate::{IndexedPolygonGenerator, PointGenerator, PolygonGenerator};
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

    fn spatial_point(&self, u: usize, v: usize) -> Point3<T> {
        let u = (T::from(u).unwrap() / T::from(self.nu).unwrap()) * T::PI() * (T::one() + T::one());
        let v = (T::from(v).unwrap() / T::from(self.nv).unwrap()) * T::PI();
        Point3::new(u.cos() * v.sin(), u.sin() * v.sin(), v.cos())
    }

    fn indexed_point(&self, u: usize, v: usize) -> usize {
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

impl<T> PointGenerator for UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    type Output = Point3<T>;

    fn spatial_point(&self, index: usize) -> Self::Output {
        if index == 0 {
            self.spatial_point(0, 0)
        }
        else if index == self.point_count() - 1 {
            self.spatial_point(0, self.nv)
        }
        else {
            let index = index - 1;
            self.spatial_point(index % self.nu, (index / self.nv) + 1)
        }
    }

    fn point_count(&self) -> usize {
        (self.nv - 1) * self.nu + 2
    }
}

impl<T> PolygonGenerator for UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    type Output = Polygon<Point3<T>>;

    fn spatial_polygon(&self, index: usize) -> Self::Output {
        let (u, v) = self.map_polygon_index(index);

        // Generate the points at the requested meridian and parallel. The
        // upper and lower bounds of (u, v) are always used, so generate them
        // in advance (`low` and `high`). Emit triangles at the poles,
        // otherwise quads.
        let low = self.spatial_point(u, v);
        let high = self.spatial_point(u + 1, v + 1);
        if v == 0 {
            Polygon::Triangle(Triangle::new(low, self.spatial_point(u, v + 1), high))
        }
        else if v == self.nv - 1 {
            Polygon::Triangle(Triangle::new(high, self.spatial_point(u + 1, v), low))
        }
        else {
            Polygon::Quad(Quad::new(
                low,
                self.spatial_point(u, v + 1),
                high,
                self.spatial_point(u + 1, v),
            ))
        }
    }

    fn polygon_count(&self) -> usize {
        self.nu * self.nv
    }
}

impl<T> IndexedPolygonGenerator for UVSphere<T>
where
    T: Float + FloatConst + Scalar,
{
    type Output = Polygon<usize>;

    fn indexed_polygon(&self, index: usize) -> <Self as IndexedPolygonGenerator>::Output {
        let (u, v) = self.map_polygon_index(index);

        let low = self.indexed_point(u, v);
        let high = self.indexed_point(u + 1, v + 1);
        if v == 0 {
            Polygon::Triangle(Triangle::new(low, self.indexed_point(u, v + 1), high))
        }
        else if v == self.nv - 1 {
            Polygon::Triangle(Triangle::new(high, self.indexed_point(u + 1, v), low))
        }
        else {
            Polygon::Quad(Quad::new(
                low,
                self.indexed_point(u, v + 1),
                high,
                self.indexed_point(u + 1, v),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::iter::FromIterator;

    use super::super::*;

    #[test]
    fn point_count() {
        assert_eq!(
            5,
            sphere::UVSphere::<f32>::with_unit_radius(3, 2)
                .spatial_points() // 5 conjoint points.
                .count()
        );
    }

    #[test]
    fn polygon_point_count() {
        assert_eq!(
            18,
            sphere::UVSphere::<f32>::with_unit_radius(3, 2)
                .spatial_polygons() // 6 triangles, 18 points.
                .points()
                .count()
        );
    }

    #[test]
    fn index_to_point_mapping() {
        assert_eq!(
            5,
            BTreeSet::from_iter(
                sphere::UVSphere::<f32>::with_unit_radius(3, 2)
                    .indexed_polygons() // 18 points, 5 indeces.
                    .points()
            ).len()
        )
    }
}
