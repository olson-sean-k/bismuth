extern crate nalgebra;
extern crate num;

use num::{One, Zero}; // TODO: `use ::std::num::{One, Zero};`.
use std::ops;

use math::{Clamp, Mask, UPoint3, UScalar, UVector3};

pub type LogWidth = u8; // TODO: https://github.com/rust-lang/rfcs/issues/671

pub const MAX_WIDTH: LogWidth = 31;
pub const MIN_WIDTH: LogWidth = 4;

impl Clamp<LogWidth> for LogWidth {
    fn clamp(&self, min: LogWidth, max: LogWidth) -> Self {
        nalgebra::clamp(*self, min, max)
    }
}

pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub fn range() -> ops::Range<usize> {
        (Axis::X as usize)..(Axis::Z as usize + 1)
    }
}

impl From<usize> for Axis {
    fn from(index: usize) -> Self {
        match index {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!() // TODO: Use `TryFrom`.
        }
    }
}

pub enum Direction {
    Positive,
    Negative,
}

pub enum Orientation {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

impl Orientation {
    pub fn axis(&self) -> Axis {
        match *self {
            Orientation::Left | Orientation::Right => Axis::X,
            Orientation::Top | Orientation::Bottom => Axis::Y,
            Orientation::Front | Orientation::Back => Axis::Z,
        }
    }

    pub fn direction(&self) -> Direction {
        match *self {
            Orientation::Left | Orientation::Top | Orientation::Front => Direction::Positive,
            Orientation::Right | Orientation::Bottom | Orientation::Back => Direction::Negative,
        }
    }
}

pub struct AABB {
    pub origin: UPoint3,
    pub extent: UVector3,
}

impl AABB {
    pub fn new(origin: UPoint3, extent: UVector3) -> Self {
        AABB {
            origin: origin,
            extent: extent,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        for axis in Axis::range() {
            if (self.origin[axis] + self.extent[axis]) < other.origin[axis] {
                return false;
            }
            if self.origin[axis] > (other.origin[axis] + other.extent[axis]) {
                return false;
            }
        }
        true
    }
}

/// A cubic spatial partition in the `UScalar` space. `Partition`s are
/// represented as an origin and a width.
#[derive(Clone, Copy)]
pub struct Partition {
    origin: UPoint3,
    width: LogWidth,
}

impl Partition {
    /// Constructs a `Partition` at the given point in space with the given
    /// width. The width is clamped to [`MIN_WIDTH`, `MAX_WIDTH`].
    pub fn at_point(point: &UPoint3, width: LogWidth) -> Self {
        let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        Partition {
            origin: point.mask(!UScalar::zero() << width),
            width: width,
        }
    }

    /// Constructs the sub-`Partition` at the given index. Returns `None` if
    /// there is no such `Partition`, because the minimum width has been
    /// exceeded.
    ///
    /// # Panics
    ///
    /// Panics if `index` is not within the range [0, 8).
    pub fn at_index(&self, index: usize) -> Option<Self> {
        if self.is_min_width() {
            None
        }
        else {
            let width = self.width - 1;
            Some(Partition {
                origin: self.origin + vector_at_index(index, width),
                width: width,
            })
        }
    }

    /// Gets the origin of the `Partition`.
    pub fn origin(&self) -> &UPoint3 {
        &self.origin
    }

    /// Gets the width of the `Partition`.
    pub fn width(&self) -> LogWidth {
        self.width
    }

    /// Gets the midpoint of the `Partition`.
    pub fn midpoint(&self) -> UPoint3 {
        let m = exp(self.width - 1);
        self.origin + UVector3::new(m, m, m)
    }

    pub fn extent(&self) -> UVector3 {
        (UVector3::one() * exp(self.width)) - UVector3::one()
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }

    /// Returns `true` if the `Partition` has the minimum possible width.
    pub fn is_min_width(&self) -> bool {
        self.width == MIN_WIDTH
    }
}

pub trait Spatial {
    fn partition(&self) -> &Partition;

    fn depth(&self) -> u8;

    fn aabb(&self) -> AABB {
        self.partition().aabb()
    }
}

pub fn exp(width: LogWidth) -> UScalar {
    UScalar::one() << width
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn index_at_point(point: &UPoint3, width: LogWidth) -> usize {
    ((((point.x >> width) & UScalar::one()) << 0) |
     (((point.y >> width) & UScalar::one()) << 1) |
     (((point.z >> width) & UScalar::one()) << 2)) as usize
}

pub fn vector_at_index(index: usize, width: LogWidth) -> UVector3 {
    assert!(index < 8);
    let index = index as UScalar;
    let width = exp(width);
    UVector3::new(((index >> 0) & UScalar::one()) * width,
                  ((index >> 1) & UScalar::one()) * width,
                  ((index >> 2) & UScalar::one()) * width)
}

#[cfg(test)]
mod tests {
    use super::*;
}
