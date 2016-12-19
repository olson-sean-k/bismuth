extern crate nalgebra;
extern crate num;

use cube;
use cube::{AABB, LogWidth, Partition, Spatial};
use math::{UPoint3, UVector3};
use num::{One, Zero};

pub struct Selection {
    cursor: Cursor,
}

impl Selection {
    pub fn at_cube<C: Spatial>(cube: &C) -> Self {
        Selection {
            cursor: Cursor::new(cube.partition().origin(),
                                cube.partition().width(),
                                &UVector3::zero()),
        }
    }

    pub fn at_point(point: &UPoint3, width: LogWidth) -> Self {
        let partition = Partition::at_point(point, width);
        Selection { cursor: Cursor::new(partition.origin(), partition.width(), &UVector3::zero()) }
    }

    pub fn at_cursor(cursor: &Cursor) -> Self {
        Selection { cursor: cursor.clone() }
    }

    pub fn span(&mut self, span: &UVector3) -> &mut Self {
        self.cursor.span = span.clone();
        self
    }

    pub fn span_to_point(&mut self, point: &UPoint3) -> &mut Self {
        unimplemented!()
    }

    pub fn to_cursor(&self) -> Cursor {
        self.cursor.clone()
    }

    pub fn into_cursor(self) -> Cursor {
        self.cursor
    }
}

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
        ((self.span + UVector3::one()) * cube::exp(self.width)) - UVector3::one()
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
