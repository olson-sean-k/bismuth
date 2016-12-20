#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate num;
extern crate rand;

pub mod cube;
pub mod edit;
pub mod math;
pub mod render;
pub mod resource;

pub mod prelude {
    pub use cube::Spatial;
    pub use math::*;
}
