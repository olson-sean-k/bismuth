use super::traverse::TraversalBuffer;
use super::tree::{Cube, Load, Node};

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

pub trait Midpoint {
    fn midpoint(&self) -> usize;
}

impl<T> Midpoint for Vec<T> {
    fn midpoint(&self) -> usize {
        self.len() / 2
    }
}

pub trait SplitAt: Sized {
    fn split_at(self, index: usize) -> (Self, Self);
}

impl<T> SplitAt for Vec<T> {
    fn split_at(mut self, index: usize) -> (Self, Self) {
        let right = self.split_off(index);
        (self, right)
    }
}

pub trait SplitBuffer<'a, N>: Default + IntoIterator<Item = Cube<'a, N>> + IsEmpty + Load +
                              Midpoint + SplitAt + TraversalBuffer<'a, N>
    where N: AsRef<Node>
{}

impl<'a, T, N> SplitBuffer<'a, N> for T
    where T: Default + IntoIterator<Item = Cube<'a, N>> + IsEmpty + Load + Midpoint + SplitAt +
             TraversalBuffer<'a, N>,
          N: AsRef<Node>
{}

pub enum Splitter {
    Midpoint,
    Load,
}

impl Splitter {
    pub fn split<'a, B, N>(&self, buffer: B) -> (B, Option<B>)
        where B: SplitBuffer<'a, N>,
              N: AsRef<Node>
    {
        match *self {
            Splitter::Midpoint => split_midpoint(buffer),
            Splitter::Load => split_load(buffer),
        }
    }
}

fn split_midpoint<'a, B, N>(buffer: B) -> (B, Option<B>)
    where B: IsEmpty + Midpoint + SplitAt + TraversalBuffer<'a, N>,
          N: AsRef<Node>
{
    let midpoint = buffer.midpoint();
    let (left, right) = buffer.split_at(midpoint);
    (left, if right.is_empty() { None } else { Some(right) })
}

// TODO: This is a naive greedy algorithm to split on load. This is often
//       suboptimal, and can be quite unbalanced if there is a large disparity
//       between the loads in a buffer.
fn split_load<'a, B, N>(buffer: B) -> (B, Option<B>)
    where B: Default + IntoIterator<Item = Cube<'a, N>> + IsEmpty + Load + TraversalBuffer<'a, N>,
          N: AsRef<Node>
{
    let mut left = B::default();
    let mut right = B::default();
    for cube in buffer {
        if left.load() <= right.load() {
            left.push(cube);
        }
        else {
            right.push(cube);
        }
    }
    (left, if right.is_empty() { None } else { Some(right) })
}
