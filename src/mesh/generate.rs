//! This module provides a generic iterator and traits for mapping from an
//! index to a primitive from some shape.

use std::ops::Range;

use super::primitive::Polygonal;

// A type `F` constrained to `Fn(&'a G, usize) -> P` could be used here, but it
// would not be possible to name that type for anything but functions (not
// closures).  Instead of a limited and somewhat redundant type `F`, just use
// `fn(&'a G, usize) -> P` for the member `f`.
pub struct Generate<'a, G, P>
where
    G: 'a,
{
    generator: &'a G,
    range: Range<usize>,
    f: fn(&'a G, usize) -> P,
}

impl<'a, G, P> Generate<'a, G, P>
where
    G: 'a,
{
    pub(super) fn new(generator: &'a G, range: Range<usize>, f: fn(&'a G, usize) -> P) -> Self {
        Generate {
            generator: generator,
            range: range,
            f: f,
        }
    }
}

impl<'a, G, P> Iterator for Generate<'a, G, P>
where
    G: 'a,
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|index| (self.f)(self.generator, index))
    }
}

pub trait ConjointPointGenerator {
    type Output;

    fn conjoint_point(&self, index: usize) -> Self::Output;
    fn conjoint_point_count(&self) -> usize;
}

pub trait ConjointPoints<P>: Sized {
    fn conjoint_points(&self) -> Generate<Self, P>;
}

impl<G, P> ConjointPoints<P> for G
where
    G: ConjointPointGenerator<Output = P>,
{
    fn conjoint_points(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.conjoint_point_count(), G::conjoint_point)
    }
}

pub trait PolygonGenerator {
    type Output: Polygonal;

    fn polygon(&self, index: usize) -> Self::Output;
    fn polygon_count(&self) -> usize;
}

pub trait Polygons<P>: Sized {
    fn polygons(&self) -> Generate<Self, P>;
}

impl<G, P> Polygons<P> for G
where
    G: PolygonGenerator<Output = P>,
    P: Polygonal,
{
    fn polygons(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.polygon_count(), G::polygon)
    }
}

pub trait IndexPolygonGenerator: ConjointPointGenerator + PolygonGenerator {
    type Output: Polygonal;

    fn index_polygon(&self, index: usize) -> <Self as IndexPolygonGenerator>::Output;
}

pub trait IndexPolygons<P>: Sized {
    fn index_polygons(&self) -> Generate<Self, P>;
}

impl<G, P> IndexPolygons<P> for G
where
    G: IndexPolygonGenerator<Output = P> + ConjointPointGenerator + PolygonGenerator,
    P: Polygonal,
{
    fn index_polygons(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.polygon_count(), G::index_polygon)
    }
}

pub trait TexturePolygonGenerator: PolygonGenerator {
    type Output: Polygonal;

    fn texture_polygon(&self, index: usize) -> <Self as TexturePolygonGenerator>::Output;
}

pub trait TexturePolygons<P>: Sized {
    fn texture_polygons(&self) -> Generate<Self, P>;
}

impl<G, P> TexturePolygons<P> for G
where
    G: PolygonGenerator + TexturePolygonGenerator<Output = P>,
    P: Polygonal,
{
    fn texture_polygons(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.polygon_count(), G::texture_polygon)
    }
}
