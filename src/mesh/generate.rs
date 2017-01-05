use std::marker::PhantomData;
use std::ops;

pub trait Conjoint<T>: Sized {
    fn conjoint_point(&self, index: usize) -> T;
    fn conjoint_point_count(&self) -> usize;
    fn conjoint_points<'a>(&'a self) -> ConjointPointIter<'a, Self, T> {
        ConjointPointIter::new(self, 0..self.conjoint_point_count())
    }
}

pub struct ConjointPointIter<'a, S: 'a, T> {
    shape: &'a S,
    points: ops::Range<usize>,
    phantom_t: PhantomData<T>,
}

impl<'a, S, T> ConjointPointIter<'a, S, T> {
    fn new(shape: &'a S, points: ops::Range<usize>) -> Self {
        ConjointPointIter {
            shape: shape,
            points: points,
            phantom_t: PhantomData,
        }
    }
}

impl<'a, S, T> Iterator for ConjointPointIter<'a, S, T>
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
    fn indexed_polygons<'a>(&'a self) -> IndexedPolygonIter<'a, Self, P> {
        IndexedPolygonIter::new(self, 0..self.indexed_polygon_count())
    }
}

pub struct IndexedPolygonIter<'a, S: 'a, P> {
    shape: &'a S,
    polygons: ops::Range<usize>,
    phantom_p: PhantomData<P>,
}

impl<'a, S, P> IndexedPolygonIter<'a, S, P> {
    fn new(shape: &'a S, polygons: ops::Range<usize>) -> Self {
        IndexedPolygonIter {
            shape: shape,
            polygons: polygons,
            phantom_p: PhantomData,
        }
    }
}

impl<'a, S, P> Iterator for IndexedPolygonIter<'a, S, P>
    where S: Indexed<P>
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.shape.indexed_polygon(index))
    }
}
