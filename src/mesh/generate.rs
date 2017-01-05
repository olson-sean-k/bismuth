use std::marker::PhantomData;
use std::ops;

pub trait Conjoint<T>: Sized {
    fn conjoint_point(&self, index: usize) -> T;
    fn conjoint_point_count(&self) -> usize;
    fn conjoint_points<'a>(&'a self) -> ConjoinPoint<'a, Self, T> {
        ConjoinPoint::new(self, 0..self.conjoint_point_count())
    }
}

pub struct ConjoinPoint<'a, S: 'a, T> {
    shape: &'a S,
    points: ops::Range<usize>,
    phantom_t: PhantomData<T>,
}

impl<'a, S, T> ConjoinPoint<'a, S, T> {
    fn new(shape: &'a S, points: ops::Range<usize>) -> Self {
        ConjoinPoint {
            shape: shape,
            points: points,
            phantom_t: PhantomData,
        }
    }
}

impl<'a, S, T> Iterator for ConjoinPoint<'a, S, T>
    where S: Conjoint<T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|index| self.shape.conjoint_point(index))
    }
}

pub trait Indexed<P>: Sized {
    fn indexed_polygon(&self, index: usize) -> P;
    fn indexed_polygon_count(&self) -> usize;
    fn indexed_polygons<'a>(&'a self) -> IndexPolygon<'a, Self, P> {
        IndexPolygon::new(self, 0..self.indexed_polygon_count())
    }
}

pub struct IndexPolygon<'a, S: 'a, P> {
    shape: &'a S,
    polygons: ops::Range<usize>,
    phantom_p: PhantomData<P>,
}

impl<'a, S, P> IndexPolygon<'a, S, P> {
    fn new(shape: &'a S, polygons: ops::Range<usize>) -> Self {
        IndexPolygon {
            shape: shape,
            polygons: polygons,
            phantom_p: PhantomData,
        }
    }
}

impl<'a, S, P> Iterator for IndexPolygon<'a, S, P>
    where S: Indexed<P>
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.shape.indexed_polygon(index))
    }
}
