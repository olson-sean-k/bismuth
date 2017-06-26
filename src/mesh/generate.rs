use std::ops::Range;

pub struct Generate<'a, G, P, F>
where
    G: 'a,
    F: Fn(&'a G, usize) -> P,
{
    generator: &'a G,
    range: Range<usize>,
    f: F,
}

impl<'a, G, P, F> Generate<'a, G, P, F>
where
    G: 'a,
    F: Fn(&'a G, usize) -> P,
{
    pub(super) fn new(generator: &'a G, range: Range<usize>, f: F) -> Self {
        Generate {
            generator: generator,
            range: range,
            f: f,
        }
    }
}

impl<'a, G, P, F> Iterator for Generate<'a, G, P, F>
where
    G: 'a,
    F: Fn(&'a G, usize) -> P,
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|index| (self.f)(&self.generator, index))
    }
}

pub trait ConjointPointGenerator<P> {
    fn conjoint_point(&self, index: usize) -> P;
    fn conjoint_point_count(&self) -> usize;
}

pub trait ConjointPoints<P>: Sized {
    fn conjoint_points<'a>(&'a self) -> Generate<'a, Self, P, fn(&'a Self, usize) -> P>;
}

impl<G, P> ConjointPoints<P> for G
where
    G: ConjointPointGenerator<P>,
{
    fn conjoint_points<'a>(&'a self) -> Generate<'a, Self, P, fn(&'a Self, usize) -> P> {
        Generate::new(self, 0..self.conjoint_point_count(), map_conjoint_point)
    }
}

pub trait PolygonGenerator {
    fn polygon_count(&self) -> usize;
}

pub trait IndexPolygonGenerator<P>: PolygonGenerator {
    fn index_polygon(&self, index: usize) -> P;
}

pub trait IndexPolygons<P>: Sized {
    fn index_polygons<'a>(&'a self) -> Generate<'a, Self, P, fn(&'a Self, usize) -> P>;
}

impl<G, P> IndexPolygons<P> for G
where
    G: IndexPolygonGenerator<P> + PolygonGenerator,
{
    fn index_polygons<'a>(&'a self) -> Generate<'a, Self, P, fn(&'a Self, usize) -> P> {
        Generate::new(self, 0..self.polygon_count(), map_index_polygon)
    }
}

pub trait TexturePolygonGenerator<P>: PolygonGenerator {
    fn texture_polygon(&self, index: usize) -> P;
}

pub trait TexturePolygons<P>: Sized {
    fn texture_polygons<'a>(&'a self) -> Generate<'a, Self, P, fn(&'a Self, usize) -> P>;
}

impl<G, P> TexturePolygons<P> for G
where
    G: PolygonGenerator + TexturePolygonGenerator<P>,
{
    fn texture_polygons<'a>(&'a self) -> Generate<'a, Self, P, fn(&'a Self, usize) -> P> {
        Generate::new(self, 0..self.polygon_count(), map_texture_polygon)
    }
}

fn map_conjoint_point<G, P>(generator: &G, index: usize) -> P
where
    G: ConjointPointGenerator<P>,
{
    generator.conjoint_point(index)
}

fn map_index_polygon<G, P>(generator: &G, index: usize) -> P
where
    G: IndexPolygonGenerator<P>,
{
    generator.index_polygon(index)
}

fn map_texture_polygon<G, P>(generator: &G, index: usize) -> P
where
    G: TexturePolygonGenerator<P>,
{
    generator.texture_polygon(index)
}
