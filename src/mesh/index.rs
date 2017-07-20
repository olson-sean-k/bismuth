// TODO: Naming is hard. `Index` is a bit generic and conflicts with the
//       `Index` type from the `render` module. Moreover, the name `index` in
//       iterator expressions is generic, but the obvious alternative
//       `index_polygons` conflicts with the `generate` module.

use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;

use render::{self, MeshBuffer, Vertex};
use super::decompose::IntoPoints;
use super::primitive::Primitive;

pub trait Indexer<T, K>: Default
where
    T: IntoPoints + Primitive,
{
    fn index<F>(&mut self, point: T::Point, f: F) -> (usize, Option<T::Point>)
    where
        F: Fn(&T::Point) -> &K;
}

pub struct HashIndexer<T, K>
where
    T: IntoPoints + Primitive,
    K: Clone + Eq + Hash,
{
    hash: HashMap<K, usize>,
    n: usize,
    phantom: PhantomData<T>,
}

impl<T, K> Default for HashIndexer<T, K>
where
    T: IntoPoints + Primitive,
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        HashIndexer {
            hash: HashMap::new(),
            n: 0,
            phantom: PhantomData,
        }
    }
}

impl<T, K> Indexer<T, K> for HashIndexer<T, K>
where
    T: IntoPoints + Primitive,
    K: Clone + Eq + Hash,
{
    fn index<F>(&mut self, input: T::Point, f: F) -> (usize, Option<T::Point>)
    where
        F: Fn(&T::Point) -> &K,
    {
        let mut point = None;
        let mut n = self.n;
        let index = self.hash.entry(f(&input).clone()).or_insert_with(|| {
            point = Some(input);
            let m = n;
            n = n + 1;
            m
        });
        self.n = n;
        (*index, point)
    }
}

pub trait Index<T>: Sized
where
    T: IntoPoints + Primitive,
{
    fn index_with_key<I, K, F>(self, f: F) -> (Vec<usize>, Vec<T::Point>)
    where
        I: Indexer<T, K>,
        F: Fn(&T::Point) -> &K;

    fn index<I>(self) -> (Vec<usize>, Vec<T::Point>)
    where
        I: Indexer<T, T::Point>,
    {
        self.index_with_key::<I, T::Point, _>(|point| point)
    }
}

impl<T, I> Index<T> for I
where
    I: Iterator<Item = T>,
    T: IntoPoints + Primitive,
{
    fn index_with_key<J, K, F>(self, f: F) -> (Vec<usize>, Vec<T::Point>)
    where
        J: Indexer<T, K>,
        F: Fn(&T::Point) -> &K,
    {
        let mut indexer = J::default();
        let mut indeces = Vec::new();
        let mut points = Vec::new();
        for primitive in self {
            for point in primitive.into_points() {
                let (index, point) = indexer.index(point, &f);
                indeces.push(index);
                if let Some(point) = point {
                    points.push(point);
                }
            }
        }
        (indeces, points)
    }
}

// TODO: This won't build, because `Vertex` is not `Eq` or `Hash` and contains
//       floating point values.
// This allows for streams of polygons containing `Vertex`s to be `collect`ed
// into a `MeshBuffer`. This is a bit dubious; the high cost and complexity is
// hidden behind an innocuous `collect` invocation.
impl<T> FromIterator<T> for MeshBuffer
where
    T: IntoPoints + Primitive<Point = Vertex>,
{
    fn from_iter<I>(input: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut buffer = MeshBuffer::new();
        let (indeces, points) = input.into_iter().index::<HashIndexer<_, _>>();
        buffer.extend(
            points,
            indeces.into_iter().map(|index| index as render::Index),
        );
        buffer
    }
}
