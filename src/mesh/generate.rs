use std::marker::PhantomData;
use std::ops::Range;

pub trait ConjointPointGenerator<T> {
    fn conjoint_point(&self, index: usize) -> T;
    fn conjoint_point_count(&self) -> usize;
}

pub trait ConjointPoints<T>: Sized {
    fn conjoint_points<'a>(&'a self) -> ConjointPointIter<'a, Self, T>;
}

impl<T, U> ConjointPoints<U> for T
    where T: ConjointPointGenerator<U>
{
    fn conjoint_points<'a>(&'a self) -> ConjointPointIter<'a, Self, U> {
        ConjointPointIter::new(self, 0..self.conjoint_point_count())
    }
}

pub struct ConjointPointIter<'a, G: 'a, T> {
    generator: &'a G,
    points: Range<usize>,
    phantom: PhantomData<T>,
}

impl<'a, G, T> ConjointPointIter<'a, G, T> {
    fn new(generator: &'a G, points: Range<usize>) -> Self {
        ConjointPointIter {
            generator: generator,
            points: points,
            phantom: PhantomData,
        }
    }
}

impl<'a, G, T> Iterator for ConjointPointIter<'a, G, T>
    where G: ConjointPointGenerator<T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.points.next().map(|index| self.generator.conjoint_point(index))
    }
}

pub trait PolygonGenerator {
    fn polygon_count(&self) -> usize;
}

pub trait IndexPolygonGenerator<P>: PolygonGenerator {
    fn index_polygon(&self, index: usize) -> P;
}

pub trait IndexPolygons<P>: Sized {
    fn index_polygons<'a>(&'a self) -> IndexPolygonIter<'a, Self, P>;
}

impl<T, P> IndexPolygons<P> for T
    where T: IndexPolygonGenerator<P> + PolygonGenerator
{
    fn index_polygons<'a>(&'a self) -> IndexPolygonIter<'a, Self, P> {
        IndexPolygonIter::new(self, 0..self.polygon_count())
    }
}

pub struct IndexPolygonIter<'a, G: 'a, P> {
    generator: &'a G,
    polygons: Range<usize>,
    phantom: PhantomData<P>,
}

impl<'a, G, P> IndexPolygonIter<'a, G, P> {
    fn new(generator: &'a G, polygons: Range<usize>) -> Self {
        IndexPolygonIter {
            generator: generator,
            polygons: polygons,
            phantom: PhantomData,
        }
    }
}

impl<'a, G, P> Iterator for IndexPolygonIter<'a, G, P>
    where G: IndexPolygonGenerator<P> + PolygonGenerator
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.generator.index_polygon(index))
    }
}

pub trait TexturePolygonGenerator<P>: PolygonGenerator {
    fn texture_polygon(&self, index: usize) -> P;
}

pub trait TexturePolygons<P>: Sized {
    fn texture_polygons<'a>(&'a self) -> TexturePolygonIter<'a, Self, P>;
}

impl<T, P> TexturePolygons<P> for T
    where T: PolygonGenerator + TexturePolygonGenerator<P>
{
    fn texture_polygons<'a>(&'a self) -> TexturePolygonIter<'a, Self, P> {
        TexturePolygonIter::new(self, 0..self.polygon_count())
    }
}

pub struct TexturePolygonIter<'a, G: 'a, P> {
    generator: &'a G,
    polygons: Range<usize>,
    phantom: PhantomData<P>,
}

impl<'a, G, P> TexturePolygonIter<'a, G, P> {
    fn new(generator: &'a G, polygons: Range<usize>) -> Self {
        TexturePolygonIter {
            generator: generator,
            polygons: polygons,
            phantom: PhantomData,
        }
    }
}

impl<'a, G, P> Iterator for TexturePolygonIter<'a, G, P>
    where G: PolygonGenerator + TexturePolygonGenerator<P>
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.polygons.next().map(|index| self.generator.texture_polygon(index))
    }
}
