extern crate nalgebra;
extern crate num;

use cube::*;
use math::FromDomain;
use num::{One, Zero};

pub struct Cursor {
    origin: Point3,
    width: LogWidth,
    span: Vector3,
}

impl Cursor {
    pub fn at_cube<C: Spatial>(cube: &C) -> Self {
        Cursor {
            origin: cube.partition().origin().clone(),
            width: cube.partition().width(),
            span: Vector3::zero(),
        }
    }

    pub fn at_cube_with_span<C: Spatial>(cube: &C, span: &Vector3) -> Self {
        let mut cursor = Cursor::at_cube(cube);
        cursor.span = span.clone();
        cursor
    }

    pub fn at_point(point: &Point3, width: LogWidth) -> Self {
        let partition = Partition::at_point(point, width);
        Cursor {
            origin: partition.origin().clone(),
            width: partition.width(),
            span: Vector3::zero(),
        }
    }

    pub fn at_point_with_span(point: &Point3, width: LogWidth, span: &Vector3) -> Self {
        let mut cursor = Cursor::at_point(point, width);
        cursor.span = span.clone();
        cursor
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

    pub fn span_mut(&mut self) -> &mut Vector3 {
        &mut self.span
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
