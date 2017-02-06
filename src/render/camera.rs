use glutin::Window;
use nalgebra::{Isometry3, Perspective3};

use math::{FMatrix4, FPoint3, FScalar, FVector3};

lazy_static! {
    static ref UP: FVector3 = FVector3::new(0.0, 0.0, 1.0);
}

pub trait AspectRatio {
    fn aspect_ratio(&self) -> FScalar;
}

impl AspectRatio for Window {
    fn aspect_ratio(&self) -> FScalar {
        let (width, height) = self.get_inner_size_pixels().unwrap_or((1, 1));
        width as FScalar / height as FScalar
    }
}

pub struct Projection {
    pub fov: FScalar,
    pub near: FScalar,
    pub far: FScalar,
}

impl Projection {
    pub fn new(fov: FScalar, near: FScalar, far: FScalar) -> Self {
        Projection {
            fov: fov,
            near: near,
            far: far,
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Projection::new(1.0, -1.0, 1.0)
    }
}

pub struct Camera {
    projection: Perspective3<FScalar>,
    view: Isometry3<FScalar>,
}

impl Camera {
    pub fn new<W>(window: &W, projection: &Projection) -> Self
        where W: AspectRatio
    {
        Camera {
            projection: Perspective3::new(window.aspect_ratio(),
                                          projection.fov,
                                          projection.near,
                                          projection.far),
            view: Isometry3::look_at_rh(&FPoint3::origin(), &FPoint3::new(0.0, 0.0, 1.0), &UP),
        }
    }

    pub fn look_at(&mut self, position: &FPoint3, point: &FPoint3) {
        self.view = Isometry3::look_at_rh(position, point, &UP);
    }

    pub fn transform(&self) -> FMatrix4 {
        self.projection() * self.view()
    }

    fn projection(&self) -> &FMatrix4 {
        self.projection.as_matrix()
    }

    fn view(&self) -> FMatrix4 {
        self.view.to_homogeneous()
    }
}
