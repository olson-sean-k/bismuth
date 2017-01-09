extern crate nalgebra;
extern crate num;

use cube::{AABB, LogWidth, Partition, Spatial};
use math::{UPoint3, UVector3};
use num::{One, Zero};

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

    pub fn at_cube<C: Spatial>(cube: &C) -> Self {
        Cursor::new(cube.partition().origin(), cube.partition().width(), &UVector3::zero())
    }

    pub fn at_point(point: &UPoint3, width: LogWidth) -> Self {
        let partition = Partition::at_point(point, width);
        Cursor::new(partition.origin(), width, &UVector3::zero())
    }

    pub fn span_from_point(point: &UPoint3, span: &UVector3, width: LogWidth) -> Self {
        let partition = Partition::at_point(point, width);
        Cursor::new(partition.origin(), width, span)
    }

    pub fn span_from_point_to_point(start: &UPoint3, end: &UPoint3, width: LogWidth) -> Self {
        let (start, end) = {
            (Partition::at_point(&UPoint3::new(nalgebra::min(start.x, end.x),
                                               nalgebra::min(start.y, end.y),
                                               nalgebra::min(start.z, end.z)),
                                 width),
             Partition::at_point(&UPoint3::new(nalgebra::max(start.x, end.x),
                                               nalgebra::max(start.y, end.y),
                                               nalgebra::max(start.z, end.z)),
                                 width))
        };
        let span = (end.origin().to_vector() - start.origin().to_vector()) / width.exp();
        Cursor::new(start.origin(), width, &span)
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
