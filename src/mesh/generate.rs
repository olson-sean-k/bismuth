//! This module provides a generic iterator and traits for mapping from an
//! index to a primitive from some shape.

use std::ops::Range;

use super::primitive::Polygonal;

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

pub trait PointGenerator {
    type Output;

    // TODO: Should this have its own `SpatialPointGenerator` trait?
    fn spatial_point(&self, index: usize) -> Self::Output;
    fn point_count(&self) -> usize;
}

pub trait SpatialPoints<P>: Sized {
    fn spatial_points(&self) -> Generate<Self, P>;
}

impl<G, P> SpatialPoints<P> for G
where
    G: PointGenerator<Output = P>,
{
    fn spatial_points(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.point_count(), G::spatial_point)
    }
}

pub trait PolygonGenerator {
    type Output: Polygonal;

    // TODO: Should this have its own `SpatialPolygonGenerator` trait?
    fn spatial_polygon(&self, index: usize) -> Self::Output;
    fn polygon_count(&self) -> usize;
}

pub trait SpatialPolygons<P>: Sized {
    fn spatial_polygons(&self) -> Generate<Self, P>;
}

impl<G, P> SpatialPolygons<P> for G
where
    G: PolygonGenerator<Output = P>,
    P: Polygonal,
{
    fn spatial_polygons(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.polygon_count(), G::spatial_polygon)
    }
}

pub trait IndexedPolygonGenerator: PointGenerator + PolygonGenerator {
    type Output: Polygonal;

    fn indexed_polygon(&self, index: usize) -> <Self as IndexedPolygonGenerator>::Output;
}

pub trait IndexedPolygons<P>: Sized {
    fn indexed_polygons(&self) -> Generate<Self, P>;
}

impl<G, P> IndexedPolygons<P> for G
where
    G: IndexedPolygonGenerator<Output = P> + PointGenerator + PolygonGenerator,
    P: Polygonal,
{
    fn indexed_polygons(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.polygon_count(), G::indexed_polygon)
    }
}

pub trait TexturedPolygonGenerator: PolygonGenerator {
    type Output: Polygonal;

    fn textured_polygon(&self, index: usize) -> <Self as TexturedPolygonGenerator>::Output;
}

pub trait TexturedPolygons<P>: Sized {
    fn textured_polygons(&self) -> Generate<Self, P>;
}

impl<G, P> TexturedPolygons<P> for G
where
    G: PolygonGenerator + TexturedPolygonGenerator<Output = P>,
    P: Polygonal,
{
    fn textured_polygons(&self) -> Generate<Self, P> {
        Generate::new(self, 0..self.polygon_count(), G::textured_polygon)
    }
}
