#![allow(unknown_lints)] // Allow clippy lints.

extern crate alga;
extern crate arrayvec;
extern crate decorum;
#[macro_use]
extern crate failure;
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
extern crate plexus;
extern crate rand;
extern crate winit;

pub mod clamp;
pub mod cube;
pub mod event;
pub mod framework;
pub mod input;
pub mod math;
pub mod render;
pub mod resource;

pub trait BoolExt: Sized {
    fn into_some<T>(self, some: T) -> Option<T>;
}

impl BoolExt for bool {
    fn into_some<T>(self, some: T) -> Option<T> {
        if self {
            Some(some)
        }
        else {
            None
        }
    }
}

pub trait OptionExt<T> {
    fn and_if<F>(self, f: F) -> Self
    where
        F: Fn(&T) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn and_if<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        match self.take() {
            Some(value) => if f(&value) {
                Some(value)
            }
            else {
                None
            },
            _ => None,
        }
    }
}
