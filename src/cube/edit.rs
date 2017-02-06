use num::Zero;
use std::cmp;

use math::{LowerBound, UPoint3, UpperBound, UVector3};
use super::space::{AABB, LogWidth, Partition, Spatial};

/// A contiguous selection of cubes in a tree.
///
/// A `Cursor` is essentially a bounding box with an associated width
/// (granularity) used to isolate cubes in a tree.
#[derive(Clone)]
pub struct Cursor {
    /// Location of the `Cursor`. Being in the `UScalar` space, `Cursor`s must
    /// extend positively from their origin. The origin is aligned to the width
    /// of the `Cursor`.
    origin: UPoint3,
    /// Target width of the `Cursor`. This is the granularity of the `Cursor`,
    /// and determines the width of the cubes on which it operates.
    width: LogWidth,
    /// The span of the `Cursor` from its origin. This is based on the width of
    /// the `Cursor`, and each component essentially selects a number of cubes
    /// in the positive direction along that axis. The zero vector means only
    /// one cube is selected by the `Cursor`.
    span: UVector3,
}

impl Cursor {
    fn new(origin: &UPoint3, width: LogWidth, span: &UVector3) -> Self {
        Cursor {
            origin: origin.clone(),
            width: width,
            span: span.clone(),
        }
    }

    pub fn at_point(point: &UPoint3, width: LogWidth) -> Self {
        Cursor::at_point_with_span(point, width, &UVector3::zero())
    }

    pub fn at_point_with_span(point: &UPoint3, width: LogWidth, span: &UVector3) -> Self {
        let partition = Partition::at_point(point, width);
        Cursor::new(partition.origin(), width, span)
    }

    pub fn from_point_to_point(start: &UPoint3, end: &UPoint3, width: LogWidth) -> Self {
        let (start, end) = {
            (Partition::at_point(&start.lower_bound(end), width),
             Partition::at_point(&start.upper_bound(end), width))
        };
        let span = (end.origin() - start.origin()) / width.exp();
        Cursor::new(start.origin(), width, &span)
    }

    pub fn at_cube<C>(cube: &C) -> Self
        where C: Spatial
    {
        Cursor::at_point(cube.partition().origin(), cube.partition().width())
    }

    pub fn at_cube_with_span<C>(cube: &C, span: &UVector3) -> Self
        where C: Spatial
    {
        Cursor::at_point_with_span(cube.partition().origin(), cube.partition().width(), span)
    }

    pub fn from_cube_to_cube<S, E>(start: &S, end: &E) -> Self
        where S: Spatial,
              E: Spatial
    {
        let width = cmp::min(start.partition().width(), end.partition().width());
        let aabb = start.aabb().union(&end.aabb());
        Cursor::from_point_to_point(&aabb.origin, &aabb.endpoint(), width)
    }

    /// Gets the origin of the `Cursor`.
    pub fn origin(&self) -> &UPoint3 {
        &self.origin
    }

    /// Gets the width associated with the `Cursor`, which is the width of the
    /// cubes that it selects.
    pub fn width(&self) -> LogWidth {
        self.width
    }

    /// Gets the span of the `Cursor` from its origin. This is based on the
    /// width of the `Cursor`, and each component essentially selects a number
    /// of cubes in the positive direction along that axis. A span of zero means
    /// that only one cube is selected by the `Cursor`.
    pub fn span(&self) -> &UVector3 {
        &self.span
    }

    /// Gets the extent of the `Cursor` from its origin.
    pub fn extent(&self) -> UVector3 {
        ((self.span + UVector3::new(1, 1, 1)) * self.width.exp()) - UVector3::new(1, 1, 1)
    }

    /// Gets the `AABB` of the `Cursor`.
    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
