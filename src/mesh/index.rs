use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;

use render::{self, MeshBuffer, Vertex};
use super::decompose::IntoPoints;
use super::primitive::Primitive;

pub trait Indexer<T, K>
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

impl<T, K> HashIndexer<T, K>
where
    T: IntoPoints + Primitive,
    K: Clone + Eq + Hash,
{
    fn new() -> Self {
        HashIndexer {
            hash: HashMap::new(),
            n: 0,
            phantom: PhantomData,
        }
    }
}

impl<T, K> Default for HashIndexer<T, K>
where
    T: IntoPoints + Primitive,
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        HashIndexer::new()
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
            n += 1;
            m
        });
        self.n = n;
        (*index, point)
    }
}

pub trait IndexPrimitives<T>: Sized
where
    T: IntoPoints + Primitive,
{
    fn index_primitives_with_key<N, K, F>(self, indexer: N, f: F) -> (Vec<usize>, Vec<T::Point>)
    where
        N: Indexer<T, K>,
        F: Fn(&T::Point) -> &K;

    fn index_primitives<N>(self, indexer: N) -> (Vec<usize>, Vec<T::Point>)
    where
        N: Indexer<T, T::Point>,
    {
        self.index_primitives_with_key::<N, T::Point, _>(indexer, |point| point)
    }
}

impl<T, I> IndexPrimitives<T> for I
where
    I: Iterator<Item = T>,
    T: IntoPoints + Primitive,
{
    fn index_primitives_with_key<N, K, F>(self, mut indexer: N, f: F) -> (Vec<usize>, Vec<T::Point>)
    where
        N: Indexer<T, K>,
        F: Fn(&T::Point) -> &K,
    {
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
        // TODO: This won't build, because `Vertex` is not `Eq` or `Hash` and
        //       contains floating point values.
        let (indeces, points) = input.into_iter().index_primitives(HashIndexer::default());
        buffer.extend(
            points,
            indeces.into_iter().map(|index| index as render::Index),
        );
        buffer
    }
}
