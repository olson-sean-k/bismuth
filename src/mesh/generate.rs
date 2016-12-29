use std::marker::PhantomData;
use std::ops;

pub trait Conjoint<T>: Sized {
    fn conjoint_point(&self, index: usize) -> T;
    fn conjoint_point_count(&self) -> usize;
    fn conjoint_points<'a>(&'a self) -> ConjointPoint<'a, T, Self> {
        ConjointPoint::new(self, 0..self.conjoint_point_count())
    }
}

pub struct ConjointPoint<'a, T, S: 'a> {
    shape: &'a S,
    points: ops::Range<usize>,
    phantom_t: PhantomData<T>,
}

impl<'a, T, S> ConjointPoint<'a, T, S> {
    fn new(shape: &'a S, points: ops::Range<usize>) -> Self {
        ConjointPoint {
            shape: shape,
            points: points,
            phantom_t: PhantomData,
        }
    }
}

impl<'a, T, S> Iterator for ConjointPoint<'a, T, S>
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
    fn indexed_polygons<'a>(&'a self) -> IndexPolygon<'a, P, Self> {
        IndexPolygon::new(self, 0..self.indexed_polygon_count())
    }
}

pub struct IndexPolygon<'a, P, S: 'a> {
    shape: &'a S,
    polygons: ops::Range<usize>,
    phantom_p: PhantomData<P>,
}

impl<'a, P, S> IndexPolygon<'a, P, S> {
    fn new(shape: &'a S, polygons: ops::Range<usize>) -> Self {
        IndexPolygon {
            shape: shape,
            polygons: polygons,
            phantom_p: PhantomData,
        }
    }
}

impl<'a, P, S> Iterator for IndexPolygon<'a, P, S>
    where S: Indexed<P>
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.shape.indexed_polygon(index))
    }
}
