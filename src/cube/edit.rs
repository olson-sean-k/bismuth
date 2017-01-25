use num::{One, Zero};
use std::cmp;

use math::{LowerBound, UPoint3, UpperBound, UVector3};
use super::space::{AABB, LogWidth, Partition, Spatial};

#[derive(Clone)]
pub struct Cursor {
    origin: UPoint3,
    width: LogWidth,
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
        let partition = Partition::at_point(point, width);
        Cursor::new(partition.origin(), width, &UVector3::zero())
    }

    pub fn span_from_point(point: &UPoint3, span: &UVector3, width: LogWidth) -> Self {
        let partition = Partition::at_point(point, width);
        Cursor::new(partition.origin(), width, span)
    }

    pub fn span_points(start: &UPoint3, end: &UPoint3, width: LogWidth) -> Self {
        let (start, end) = {
            (Partition::at_point(&start.lower_bound(end), width),
             Partition::at_point(&start.upper_bound(end), width))
        };
        let span = (end.origin().to_vector() - start.origin().to_vector()) / width.exp();
        Cursor::new(start.origin(), width, &span)
    }

    pub fn at_cube<C>(cube: &C) -> Self
        where C: Spatial
    {
        Cursor::at_point(cube.partition().origin(), cube.partition().width())
    }

    pub fn span_from_cube<C>(cube: &C, span: &UVector3) -> Self
        where C: Spatial
    {
        Cursor::span_from_point(cube.partition().origin(), span, cube.partition().width())
    }

    pub fn span_cubes<S, E>(start: &S, end: &E) -> Self
        where S: Spatial,
              E: Spatial
    {
        let width = cmp::min(start.partition().width(), end.partition().width());
        let aabb = start.aabb().union(&end.aabb());
        Cursor::span_points(&aabb.origin, &aabb.endpoint(), width)
    }

    pub fn origin(&self) -> &UPoint3 {
        &self.origin
    }

    pub fn width(&self) -> LogWidth {
        self.width
    }

    pub fn span(&self) -> &UVector3 {
        &self.span
    }

    pub fn extent(&self) -> UVector3 {
        ((self.span + UVector3::one()) * self.width.exp()) - UVector3::one()
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
