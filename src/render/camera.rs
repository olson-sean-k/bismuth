use glutin::Window;
use nalgebra::{Isometry3, Perspective3};

use math::{FMatrix4, FPoint2, FPoint3, FRay3, FScalar, FVector3, UPoint2, UScalar};

lazy_static! {
    static ref UP: FVector3 = FVector3::z();
}

pub trait AspectRatio {
    fn dimensions(&self) -> (UScalar, UScalar);

    fn aspect_ratio(&self) -> FScalar {
        let (width, height) = self.dimensions();
        width as FScalar / height as FScalar
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
        Projection::new(1.0, -1.0, 1.0)
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
            view: Isometry3::look_at_rh(&FPoint3::origin(), &FPoint3::new(0.0, 0.0, 1.0), &UP),
        }
    }

    pub fn look_at(&mut self, position: &FPoint3, point: &FPoint3) {
        self.view = Isometry3::look_at_rh(position, point, &UP);
    }

    pub fn cast_ray<W>(&self, window: &W, point: &UPoint2) -> FRay3
        where W: AspectRatio
    {
        let (width, height) = window.dimensions();
        let point = FPoint2::new(point.x as FScalar / width as FScalar,
                                 point.y as FScalar / height as FScalar);
        let near = self.projection.unproject_point(&FPoint3::new(point.x, point.y, -1.0));
        let far = self.projection.unproject_point(&FPoint3::new(point.x, point.y, 1.0));
        let inverse = self.view.inverse();
        FRay3::new(inverse * near, (inverse * (far - near)))
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
