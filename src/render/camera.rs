use glutin::Window;
use nalgebra::{Isometry3, PerspectiveMatrix3, ToHomogeneous};
use num::Zero;

use math::{FMatrix4, FPoint3, FVector3};

pub struct Camera<'a> {
    window: &'a Window,
    position: FPoint3,
    view: FMatrix4,
}

impl<'a> Camera<'a> {
    pub fn new(window: &'a Window, position: &FPoint3) -> Self {
        Camera {
            window: window,
            position: position.clone(),
            view: FMatrix4::zero(),
        }
    }

    pub fn look_at(&mut self, point: &FPoint3) {
        self.view = look_at(&self.position, point);
    }

    pub fn transform(&self) -> FMatrix4 {
        project_from_window(self.window) * self.view
    }

    pub fn position(&self) -> &FPoint3 {
        &self.position
    }
}

fn project_from_window(window: &Window) -> FMatrix4 {
    let (width, height) = window.get_inner_size_pixels().unwrap();
    PerspectiveMatrix3::new(width as f32 / height as f32, 1.0, -1.0, 1.0).to_matrix()
}

fn look_at(from: &FPoint3, to: &FPoint3) -> FMatrix4 {
    Isometry3::look_at_rh(from, to, &FVector3::new(0.0, 0.0, 1.0)).to_homogeneous()
}
