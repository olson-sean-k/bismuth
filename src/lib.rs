extern crate alga;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate num;
extern crate rand;
extern crate winit;

pub mod clamp;
pub mod cube;
pub mod event;
pub mod framework;
pub mod input;
pub mod math;
pub mod mesh;
pub mod render;
pub mod resource;

pub trait OptionExt<T> {
    fn and_if<F>(self, f: F) -> Self where F: Fn(&T) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn and_if<F>(mut self, f: F) -> Self
        where F: Fn(&T) -> bool
    {
        match self.take() {
            Some(value) => if f(&value) { Some(value) } else { None },
            _ => None,
        }
    }
}
