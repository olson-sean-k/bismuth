extern crate nalgebra;
extern crate num;

use cube::*;
use math::FromDomain;
use num::One;

pub type Span3 = nalgebra::Vector3<u8>;

pub struct Cursor {
    origin: Point3,
    width: LogWidth,
    span: Span3,
}

impl Cursor {
    pub fn at_cube<C: Spatial>(cube: &C) -> Self {
        Cursor {
            origin: cube.partition().origin().clone(),
            width: cube.partition().width(),
            span: Span3::new(0, 0, 0),
        }
    }

    pub fn origin(&self) -> &Point3 {
        &self.origin
    }

    pub fn width(&self) -> LogWidth {
        self.width
    }

    pub fn span(&self) -> &Span3 {
        &self.span
    }

    pub fn extent(&self) -> Vector3 {
        (Vector3::from_domain(self.span) + Vector3::one()) * exp(self.width)
    }

    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
