use glutin::Window;
use nalgebra::{Isometry3, Perspective3};
use num::traits::FloatConst;

use event::{Event, Reactor};
use math::{FMatrix4, FPoint2, FPoint3, FRay3, FScalar, FVector3, UPoint2, UScalar};

lazy_static! {
    static ref UP: FVector3 = FVector3::y();
}

pub trait AspectRatio {
    fn dimensions(&self) -> (u32, u32);

    fn aspect_ratio(&self) -> f32 {
        let (width, height) = self.dimensions();
        width as f32 / height as f32
    }
}

impl AspectRatio for Window {
    fn dimensions(&self) -> (UScalar, UScalar) {
        self.get_inner_size_pixels().unwrap_or((1, 1))
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
        Projection::new(FScalar::FRAC_PI_2(), 0.1, 100.0)
    }
}

impl From<Perspective3<FScalar>> for Projection {
    fn from(perspective: Perspective3<FScalar>) -> Self {
        Projection::new(perspective.fovy(), perspective.znear(), perspective.zfar())
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
            view: Isometry3::look_at_rh(&FPoint3::origin(), &FPoint3::new(0.0, 0.0, -1.0), &UP),
        }
    }

    pub fn look_at(&mut self, from: &FPoint3, to: &FPoint3) {
        self.view = Isometry3::look_at_rh(from, to, &UP);
    }

    pub fn cast_ray<W>(&self, window: &W, point: &UPoint2) -> FRay3
        where W: AspectRatio
    {
        let (width, height) = window.dimensions();
        let point = FPoint2::new(((2.0 * point.x as FScalar) / width as FScalar) - 1.0,
                                 1.0 - ((2.0 * point.y as FScalar) / height as FScalar));
        let near = self.projection.unproject_point(&FPoint3::new(point.x, point.y, -1.0));
        let far = self.projection.unproject_point(&FPoint3::new(point.x, point.y, 1.0));
        let inverse = self.view.inverse();
        FRay3::new(inverse * near, (inverse * (far - near)))
    }

    pub fn transform(&self) -> FMatrix4 {
        self.projection.as_matrix() * self.view.to_homogeneous()
    }
}

impl Reactor for Camera {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::Resized(width, height) => {
                let ratio = (width as f32) / (height as f32);
                self.projection = Perspective3::new(ratio,
                                                    self.projection.fovy(),
                                                    self.projection.znear(),
                                                    self.projection.zfar());
            }
            _ => {}
        }
    }
}
