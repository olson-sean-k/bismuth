use std::ops::Range;

pub struct Generate<'a, S, T, F>
    where S: 'a,
          F: Fn(&'a S, usize) -> T
{
    source: &'a S,
    range: Range<usize>,
    f: F,
}

impl<'a, S, T, F> Generate<'a, S, T, F>
    where S: 'a,
          F: Fn(&'a S, usize) -> T
{
    pub(super) fn new(source: &'a S, range: Range<usize>, f: F) -> Self {
        Generate {
            source: source,
            range: range,
            f: f,
        }
    }
}

impl<'a, S, T, F> Iterator for Generate<'a, S, T, F>
    where S: 'a,
          F: Fn(&'a S, usize) -> T
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|index| (self.f)(&self.source, index))
    }
}

pub trait ConjointPointGenerator<T> {
    fn conjoint_point(&self, index: usize) -> T;
    fn conjoint_point_count(&self) -> usize;
}

pub trait ConjointPoints<T>: Sized {
    fn conjoint_points<'a>(&'a self) -> Generate<'a, Self, T, fn(&'a Self, usize) -> T>;
}

impl<T, U> ConjointPoints<U> for T
    where T: ConjointPointGenerator<U>
{
    fn conjoint_points<'a>(&'a self) -> Generate<'a, Self, U, fn(&'a Self, usize) -> U> {
        Generate::new(self, 0..self.conjoint_point_count(), map_conjoint_point)
    }
}

pub trait PolygonGenerator {
    fn polygon_count(&self) -> usize;
}

pub trait IndexPolygonGenerator<T>: PolygonGenerator {
    fn index_polygon(&self, index: usize) -> T;
}

pub trait IndexPolygons<T>: Sized {
    fn index_polygons<'a>(&'a self) -> Generate<'a, Self, T, fn(&'a Self, usize) -> T>;
}

impl<T, U> IndexPolygons<U> for T
    where T: IndexPolygonGenerator<U> + PolygonGenerator
{
    fn index_polygons<'a>(&'a self) -> Generate<'a, Self, U, fn(&'a Self, usize) -> U> {
        Generate::new(self, 0..self.polygon_count(), map_index_polygon)
    }
}

pub trait TexturePolygonGenerator<T>: PolygonGenerator {
    fn texture_polygon(&self, index: usize) -> T;
}

pub trait TexturePolygons<T>: Sized {
    fn texture_polygons<'a>(&'a self) -> Generate<'a, Self, T, fn(&'a Self, usize) -> T>;
}

impl<T, U> TexturePolygons<U> for T
    where T: PolygonGenerator + TexturePolygonGenerator<U>
{
    fn texture_polygons<'a>(&'a self) -> Generate<'a, Self, U, fn(&'a Self, usize) -> U> {
        Generate::new(self, 0..self.polygon_count(), map_texture_polygon)
    }
}

fn map_conjoint_point<S, T>(source: &S, index: usize) -> T
    where S: ConjointPointGenerator<T>
{
    source.conjoint_point(index)
}

fn map_index_polygon<S, T>(source: &S, index: usize) -> T
    where S: IndexPolygonGenerator<T>
{
    source.index_polygon(index)
}

fn map_texture_polygon<S, T>(source: &S, index: usize) -> T
    where S: TexturePolygonGenerator<T>
{
    source.texture_polygon(index)
}
