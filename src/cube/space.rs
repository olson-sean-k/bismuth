extern crate nalgebra;
extern crate num;

use num::{One, Zero}; // TODO: `use ::std::num::{One, Zero};`.

use math::{Clamp, Mask, DiscreteSpace};

pub type Point3 = nalgebra::Point3<DiscreteSpace>;
pub type Vector3 = nalgebra::Vector3<DiscreteSpace>;

pub type RootWidth = u8; // TODO: https://github.com/rust-lang/rfcs/issues/671

pub const MAX_WIDTH: RootWidth = 32;
pub const MIN_WIDTH: RootWidth = 4;

impl Clamp<RootWidth> for RootWidth {
    fn clamp(&self, min: RootWidth, max: RootWidth) -> Self {
        nalgebra::clamp(*self, min, max)
    }
}

pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
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

/// A cubic spatial partition in the `DiscreteSpace`. `Partition`s are
/// represented as an origin and a width.
#[derive(Clone, Copy)]
pub struct Partition {
    origin: Point3,
    width: RootWidth,
}

impl Partition {
    /// Constructs a `Partition` at the given point in space with the given
    /// width. The width is clamped to [`MIN_WIDTH`, `MAX_WIDTH`].
    pub fn at_point(point: &Point3, width: RootWidth) -> Self {
        let width = width.clamp(MIN_WIDTH, MAX_WIDTH);
        Partition {
            origin: point.mask(!DiscreteSpace::zero() << (width - 1)),
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
    pub fn origin(&self) -> &Point3 {
        &self.origin
    }

    /// Gets the width of the `Partition`.
    pub fn width(&self) -> RootWidth {
        self.width
    }

    /// Gets the midpoint of the `Partition`.
    pub fn midpoint(&self) -> Point3 {
        let m = exp(self.width - 1);
        self.origin + Vector3::new(m, m, m)
    }

    /// Returns `true` if the `Partition` has the minimum possible width.
    pub fn is_min_width(&self) -> bool {
        self.width == MIN_WIDTH
    }
}

pub trait Spatial {
    fn partition(&self) -> &Partition;

    fn depth(&self) -> u8;
}

pub fn exp(width: RootWidth) -> DiscreteSpace {
    if width > 0 {
        DiscreteSpace::one() << (width - 1)
    }
    else {
        0
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn index_at_point(point: &Point3, width: RootWidth) -> usize {
    ((((point.x >> width) & DiscreteSpace::one()) << 0) |
     (((point.y >> width) & DiscreteSpace::one()) << 1) |
     (((point.z >> width) & DiscreteSpace::one()) << 2)) as usize
}

pub fn vector_at_index(index: usize, width: RootWidth) -> Vector3 {
    assert!(index < 8);
    let index = index as DiscreteSpace;
    let width = exp(width);
    Vector3::new(((index >> 0) & DiscreteSpace::one()) * width,
                 ((index >> 1) & DiscreteSpace::one()) * width,
                 ((index >> 2) & DiscreteSpace::one()) * width)
}
