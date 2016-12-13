extern crate nalgebra;
extern crate num;

use cube::*;
use num::{One, Zero};

pub struct Selection {
    cursor: Cursor,
}

impl Selection {
    pub fn at_cube<C: Spatial>(cube: &C) -> Self {
        Selection {
            cursor: Cursor::new(cube.partition().origin(),
                                cube.partition().width(),
                                &Vector3::zero()),
        }
    }

    pub fn at_point(point: &Point3, width: LogWidth) -> Self {
        let partition = Partition::at_point(point, width);
        Selection { cursor: Cursor::new(partition.origin(), partition.width(), &Vector3::zero()) }
    }

    pub fn at_cursor(cursor: &Cursor) -> Self {
        Selection { cursor: cursor.clone() }
    }

    pub fn span(&mut self, span: &Vector3) -> &mut Self {
        self.cursor.span = span.clone();
        self
    }

    pub fn span_to_point(&mut self, point: &Point3) -> &mut Self {
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
    origin: Point3,
    width: LogWidth,
    span: Vector3,
}

impl Cursor {
    fn new(origin: &Point3, width: LogWidth, span: &Vector3) -> Self {
        Cursor {
            origin: origin.clone(),
            width: width,
            span: span.clone(),
        }
    }

    pub fn origin(&self) -> &Point3 {
        &self.origin
    }

    pub fn width(&self) -> LogWidth {
        self.width
    }

    pub fn span(&self) -> &Vector3 {
        &self.span
    }

    pub fn extent(&self) -> Vector3 {
        ((self.span + Vector3::one()) * exp(self.width)) - Vector3::one()
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
