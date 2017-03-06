use std::marker::PhantomData;
use std::ops;

pub trait ConjointGenerator<T> {
    fn conjoint_point(&self, index: usize) -> T;
    fn conjoint_point_count(&self) -> usize;
}

pub trait Conjoint<T>: Sized {
    fn conjoint_points<'a>(&'a self) -> ConjointPointIter<'a, Self, T>;
}

impl<T, U> Conjoint<U> for T
    where T: ConjointGenerator<U>
{
    fn conjoint_points<'a>(&'a self) -> ConjointPointIter<'a, Self, U> {
        ConjointPointIter::new(self, 0..self.conjoint_point_count())
    }
}

pub struct ConjointPointIter<'a, S: 'a, T> {
    shape: &'a S,
    points: ops::Range<usize>,
    phantom: PhantomData<T>,
}

impl<'a, S, T> ConjointPointIter<'a, S, T> {
    fn new(shape: &'a S, points: ops::Range<usize>) -> Self {
        ConjointPointIter {
            shape: shape,
            points: points,
            phantom: PhantomData,
        }
    }
}

impl<'a, S, T> Iterator for ConjointPointIter<'a, S, T>
    where S: ConjointGenerator<T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|index| self.shape.conjoint_point(index))
    }
}

pub trait PolygonGenerator {
    fn polygon_count(&self) -> usize;
}

pub trait IndexedGenerator<P>: PolygonGenerator {
    fn indexed_polygon(&self, index: usize) -> P;
}

pub trait Indexed<P>: Sized {
    fn indexed_polygons<'a>(&'a self) -> IndexedPolygonIter<'a, Self, P>;
}

impl<T, P> Indexed<P> for T
    where T: IndexedGenerator<P> + PolygonGenerator
{
    fn indexed_polygons<'a>(&'a self) -> IndexedPolygonIter<'a, Self, P> {
        IndexedPolygonIter::new(self, 0..self.polygon_count())
    }
}

pub struct IndexedPolygonIter<'a, S: 'a, P> {
    shape: &'a S,
    polygons: ops::Range<usize>,
    phantom: PhantomData<P>,
}

impl<'a, S, P> IndexedPolygonIter<'a, S, P> {
    fn new(shape: &'a S, polygons: ops::Range<usize>) -> Self {
        IndexedPolygonIter {
            shape: shape,
            polygons: polygons,
            phantom: PhantomData,
        }
    }
}

impl<'a, S, P> Iterator for IndexedPolygonIter<'a, S, P>
    where S: IndexedGenerator<P> + PolygonGenerator
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.shape.indexed_polygon(index))
    }
}

pub trait TexturedGenerator<P> {
    fn textured_polygon(&self, index: usize) -> P;
}

pub trait Textured<P>: Sized {
    fn textured_polygons<'a>(&'a self) -> TexturedPolygonIter<'a, Self, P>;
}

impl<T, P> Textured<P> for T
    where T: PolygonGenerator + TexturedGenerator<P>
{
    fn textured_polygons<'a>(&'a self) -> TexturedPolygonIter<'a, Self, P> {
        TexturedPolygonIter::new(self, 0..self.polygon_count())
    }
}

pub struct TexturedPolygonIter<'a, S: 'a, P> {
    shape: &'a S,
    polygons: ops::Range<usize>,
    phantom: PhantomData<P>,
}

impl<'a, S, P> TexturedPolygonIter<'a, S, P> {
    fn new(shape: &'a S, polygons: ops::Range<usize>) -> Self {
        TexturedPolygonIter {
            shape: shape,
            polygons: polygons,
            phantom: PhantomData,
        }
    }
}

impl<'a, S, P> Iterator for TexturedPolygonIter<'a, S, P>
    where S: PolygonGenerator + TexturedGenerator<P>
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.shape.textured_polygon(index))
    }
}
