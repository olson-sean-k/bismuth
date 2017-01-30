#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate num;
extern crate rand;

pub mod camera;
pub mod clamp;
pub mod cube;
pub mod math;
pub mod mesh;
pub mod render;
pub mod resource;

pub mod prelude {
    pub use cube::Spatial;
    pub use math::*;
}

pub trait OptionExt<T> {
    fn and_if<F>(self, f: F) -> Self
        where F: Fn(&T) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn and_if<F>(mut self, f: F) -> Self
        where F: Fn(&T) -> bool
    {
        match self.take() {
            Some(value) => {
                if f(&value) {
                    Some(value)
                }
                else {
                    None
                }
            },
            _ => None,
        }
    }
}
