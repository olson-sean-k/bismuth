use nalgebra::{self, Point3};
use num::Float;
use num::traits::FloatConst;
use std::iter::Peekable;
use std::ops::Range;
use std::marker::PhantomData;

use super::generate::{Conjoint, Indexed};
use super::primitive::{Polygon, Triangle, Quad};

pub struct UVSphere<T>
    where T: Float + FloatConst
{
    nu: usize,
    nv: usize,
    us: Range<usize>, // meridians
    vs: Peekable<Range<usize>>, // parallels
    phantom_t: PhantomData<T>,
}

impl<T> UVSphere<T>
    where T: Float + FloatConst
{
    pub fn with_unit_radius(nu: usize, nv: usize) -> Self {
        let nu = nalgebra::max(3, nu);
        let nv = nalgebra::max(3, nv);
        UVSphere {
            nu: nu,
            nv: nv,
            us: 0..nu,
            vs: (0..nv).peekable(),
            phantom_t: PhantomData,
        }
    }

    fn point(&self, u: usize, v: usize) -> Point3<T> {
        point(u, v, self.nu, self.nv)
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
}

impl<T> Conjoint<Point3<T>> for UVSphere<T>
    where T: Float + FloatConst
{
    fn conjoint_point(&self, index: usize) -> Point3<T> {
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

impl<T> Indexed<Polygon<usize>> for UVSphere<T>
    where T: Float + FloatConst
{
    fn indexed_polygon(&self, index: usize) -> Polygon<usize> {
        let u = index % self.nu;
        let v = index / self.nu;

        let lower = self.indexed_point(u, v);
        let upper = self.indexed_point(u + 1, v + 1);
        if v == 0 {
            Polygon::Triangle(Triangle::new(lower, self.indexed_point(u, v + 1), upper))
        }
        else if v == self.nv - 1 {
            Polygon::Triangle(Triangle::new(upper, self.indexed_point(u + 1, v), lower))
        }
        else {
            Polygon::Quad(Quad::new(lower,
                                    self.indexed_point(u, v + 1),
                                    upper,
                                    self.indexed_point(u + 1, v)))
        }
    }

    fn indexed_polygon_count(&self) -> usize {
        self.nu * self.nv
    }
}

impl<T> Iterator for UVSphere<T>
    where T: Float + FloatConst
{
    type Item = Polygon<Point3<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let nu = self.nu;
        let nv = self.nv;

        // Iterate over meridians (`self.us`). When meridians are exhausted,
        // reset the iteration and advance iteration over parallels (`self.vs`).
        // Continue until parallels are exhausted.
        let u = match self.us.next() {
            Some(u) => u,
            None => {
                self.vs.next();
                self.us = 1..nu;
                0
            },
        };
        self.vs.peek().map(|v| {
            let v = *v;

            // Generate the points at the current meridian and parallel. The
            // upper and lower bounds of (u, v) are always used, so Generate
            // them in advance. Emit triangles at the poles, otherwise quads.
            let lower = point(u, v, nu, nv);
            let upper = point(u + 1, v + 1, nu, nv);
            if v == 0 {
                Polygon::Triangle(Triangle::new(lower, point(u, v + 1, nu, nv), upper))
            }
            else if v == nv - 1 {
                Polygon::Triangle(Triangle::new(upper, point(u + 1, v, nu, nv), lower))
            }
            else {
                Polygon::Quad(Quad::new(lower,
                                        point(u, v + 1, nu, nv),
                                        upper,
                                        point(u + 1, v, nu, nv)))
            }
        })
    }
}

fn point<T>(u: usize, v: usize, nu: usize, nv: usize) -> Point3<T>
    where T: Float + FloatConst
{
    let u = (T::from(u).unwrap() / T::from(nu).unwrap()) * T::PI() * (T::one() + T::one());
    let v = (T::from(v).unwrap() / T::from(nv).unwrap()) * T::PI();
    Point3::new(u.cos() * v.sin(), u.sin() * v.sin(), v.cos())
}
