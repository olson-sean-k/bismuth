use alga::general::{Real, SupersetOf};
use nalgebra::{self, Matrix4, Point2, Point3, Scalar, Unit, Vector3, Vector4};
use num::{Float, Integer};
use std::cmp;
use std::ops;

pub type UScalar = u32;
pub type FScalar = f32;

pub type UPoint2 = Point2<UScalar>;
pub type UPoint3 = Point3<UScalar>;
pub type UVector3 = Vector3<UScalar>;

pub type FPoint2 = Point2<FScalar>;
pub type FPoint3 = Point3<FScalar>;
pub type FVector3 = Vector3<FScalar>;
pub type FVector4 = Vector4<FScalar>;
pub type FMatrix4 = Matrix4<FScalar>;

pub type FRay3 = Ray3<FScalar>;

pub struct Ray3<T>
where
    T: Scalar,
{
    pub origin: Point3<T>,
    pub direction: Unit<Vector3<T>>,
}

impl<T> Ray3<T>
where
    T: Real + Scalar,
{
    pub fn new(origin: Point3<T>, direction: Vector3<T>) -> Self {
        Ray3 {
            origin: origin,
            direction: Unit::new_normalize(direction),
        }
    }

    pub fn x() -> Self {
        Ray3::new(Point3::origin(), Vector3::x())
    }

    pub fn y() -> Self {
        Ray3::new(Point3::origin(), Vector3::y())
    }

    pub fn z() -> Self {
        Ray3::new(Point3::origin(), Vector3::z())
    }
}

pub trait Matrix4Ext<T>
where
    T: Scalar,
{
    fn to_array(&self) -> [[T; 4]; 4];
}

impl<T> Matrix4Ext<T> for Matrix4<T>
where
    T: Scalar,
{
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn to_array(&self) -> [[T; 4]; 4] {
        [
            [self[0],  self[1],  self[2],  self[3]],
            [self[4],  self[5],  self[6],  self[7]],
            [self[8],  self[9],  self[10], self[11]],
            [self[12], self[13], self[14], self[15]]
        ]
    }
}

// TODO: The `FromSpace` and `IntoSpace` traits may not be useful. Instead, the
//       `nalgebra::convert` function can be used directly.
pub trait FromSpace<T> {
    fn from_space(value: T) -> Self;
}

impl<T, U> FromSpace<Point2<U>> for Point2<T>
where
    T: SupersetOf<U> + Scalar,
    U: Scalar,
{
    fn from_space(point: Point2<U>) -> Self {
        nalgebra::convert(point)
    }
}

impl<T, U> FromSpace<Point3<U>> for Point3<T>
where
    T: SupersetOf<U> + Scalar,
    U: Scalar,
{
    fn from_space(point: Point3<U>) -> Self {
        nalgebra::convert(point)
    }
}

impl<T, U> FromSpace<Vector3<U>> for Vector3<T>
where
    T: SupersetOf<U> + Scalar,
    U: Scalar,
{
    fn from_space(vector: Vector3<U>) -> Self {
        nalgebra::convert(vector)
    }
}

pub trait IntoSpace<T> {
    fn into_space(self) -> T;
}

impl<T, U> IntoSpace<U> for T
where
    U: FromSpace<T>,
{
    fn into_space(self) -> U {
        U::from_space(self)
    }
}

pub trait Clamp<T>
where
    T: PartialOrd,
{
    fn clamp(&self, min: T, max: T) -> Self;
}

impl<T> Clamp<T> for Point3<T>
where
    T: PartialOrd + Scalar,
{
    fn clamp(&self, min: T, max: T) -> Self {
        use nalgebra::clamp;

        Point3::new(
            clamp(self.x, min, max),
            clamp(self.y, min, max),
            clamp(self.z, min, max),
        )
    }
}

pub trait Mask<T>
where
    T: ops::BitAnd<Output = T>,
{
    fn mask(&self, value: T) -> Self;
}

impl<T> Mask<T> for Point3<T>
where
    T: ops::BitAnd<Output = T> + Scalar,
{
    fn mask(&self, value: T) -> Self {
        Point3::new(self.x & value, self.y & value, self.z & value)
    }
}

pub trait UpperBound {
    fn upper_bound(&self, other: &Self) -> Self;
}

impl<T> UpperBound for Point3<T>
where
    T: Ord + Scalar,
{
    fn upper_bound(&self, other: &Self) -> Self {
        Point3::new(
            cmp::max(self.x, other.x),
            cmp::max(self.y, other.y),
            cmp::max(self.y, other.y),
        )
    }
}

pub trait LowerBound {
    fn lower_bound(&self, other: &Self) -> Self;
}

impl<T> LowerBound for Point3<T>
where
    T: Ord + Scalar,
{
    fn lower_bound(&self, other: &Self) -> Self {
        Point3::new(
            cmp::min(self.x, other.x),
            cmp::min(self.y, other.y),
            cmp::min(self.y, other.y),
        )
    }
}

pub trait Interpolate<F>: Sized
where
    F: Float,
{
    fn lerp(&self, other: &Self, f: F) -> Self;

    fn midpoint(&self, other: &Self) -> Self {
        self.lerp(other, F::one() / (F::one() + F::one()))
    }
}

impl<T, F> Interpolate<F> for (T, T)
where
    T: SupersetOf<F> + Scalar,
    F: SupersetOf<T> + Float,
{
    fn lerp(&self, other: &Self, f: F) -> Self {
        (lerp(self.0, other.0, f), lerp(self.1, other.1, f))
    }
}

impl<T, F> Interpolate<F> for (T, T, T)
where
    T: SupersetOf<F> + Scalar,
    F: SupersetOf<T> + Float,
{
    fn lerp(&self, other: &Self, f: F) -> Self {
        (
            lerp(self.0, other.0, f),
            lerp(self.1, other.1, f),
            lerp(self.2, other.2, f),
        )
    }
}

impl<T, F> Interpolate<F> for Point2<T>
where
    T: SupersetOf<F> + Scalar,
    F: SupersetOf<T> + Float,
{
    fn lerp(&self, other: &Self, f: F) -> Self {
        Point2::new(lerp(self.x, other.x, f), lerp(self.y, other.y, f))
    }
}

impl<T, F> Interpolate<F> for Point3<T>
where
    T: SupersetOf<F> + Scalar,
    F: SupersetOf<T> + Float,
{
    fn lerp(&self, other: &Self, f: F) -> Self {
        Point3::new(
            lerp(self.x, other.x, f),
            lerp(self.y, other.y, f),
            lerp(self.z, other.z, f),
        )
    }
}

pub fn lerp<T, F>(a: T, b: T, f: F) -> T
where
    T: SupersetOf<F> + Scalar,
    F: SupersetOf<T> + Float,
{
    use nalgebra::{convert, clamp};

    let f = clamp(f, F::zero(), F::one());
    let af = convert::<T, F>(a) * (F::one() - f);
    let bf = convert::<T, F>(b) * f;
    convert::<F, T>(af + bf)
}

pub fn ordered_pair<T>(a: T, b: T) -> (T, T)
where
    T: PartialOrd,
{
    if a <= b {
        (a, b)
    }
    else {
        (b, a)
    }
}

pub fn partial_min<T>(a: T, b: T) -> T
where
    T: PartialOrd,
{
    if a <= b {
        a
    }
    else {
        b
    }
}

pub fn partial_max<T>(a: T, b: T) -> T
where
    T: PartialOrd,
{
    if a > b {
        a
    }
    else {
        b
    }
}

pub fn umod<T>(n: T, m: T) -> T
where
    T: Copy + Integer,
{
    ((n % m) + m) % m
}

#[cfg(test)]
mod tests {
    use super::*;
}
