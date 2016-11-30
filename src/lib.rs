#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate num;
extern crate rand;

use std::marker;

pub mod cube;
pub mod math;
pub mod render;
pub mod resource;

pub mod prelude {
    pub use cube::Spatial;
    pub use math::*;
    pub use render::GeometricCube;
}

pub trait IgnorableResult: marker::Sized {
    fn ignore(self) {}
}

impl<T, E> IgnorableResult for Result<T, E> {}
