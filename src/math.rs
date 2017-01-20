use nalgebra;
use num::{Float, Num};
use std::cmp;
use std::ops;

pub type UScalar = u32;
pub type FScalar = f32;

pub type UPoint3 = nalgebra::Point3<UScalar>;
pub type UVector3 = nalgebra::Vector3<UScalar>;

pub type FPoint3 = nalgebra::Point3<FScalar>;
pub type FVector3 = nalgebra::Vector3<FScalar>;
pub type FVector4 = nalgebra::Vector4<FScalar>;
pub type FMatrix4 = nalgebra::Matrix4<FScalar>;

pub trait FromSpace<T> {
    fn from_space(value: T) -> Self;
}

impl<T, U> FromSpace<nalgebra::Point3<U>> for nalgebra::Point3<T>
    where T: nalgebra::Cast<U> + Copy,
          U: Copy
{
    fn from_space(point: nalgebra::Point3<U>) -> Self {
        nalgebra::cast(point)
    }
}

impl<T, U> FromSpace<nalgebra::Vector3<U>> for nalgebra::Vector3<T>
    where T: nalgebra::Cast<U> + Copy,
          U: Copy
{
    fn from_space(vector: nalgebra::Vector3<U>) -> Self {
        nalgebra::cast(vector)
    }
}

pub trait IntoSpace<T> {
    fn into_space(self) -> T;
}

impl<T, U> IntoSpace<U> for T
    where U: FromSpace<T>
{
    fn into_space(self) -> U {
        U::from_space(self)
    }
}

pub trait Clamp<T>
    where T: PartialOrd
{
    fn clamp(&self, min: T, max: T) -> Self;
}

impl<T> Clamp<T> for nalgebra::Point3<T>
    where T: Copy + PartialOrd
{
    fn clamp(&self, min: T, max: T) -> Self {
        nalgebra::Point3::new(nalgebra::clamp(self.x, min, max),
                              nalgebra::clamp(self.y, min, max),
                              nalgebra::clamp(self.z, min, max))
    }
}

pub trait Mask<T>
    where T: ops::BitAnd<Output = T>
{
    fn mask(&self, value: T) -> Self;
}

impl<T> Mask<T> for nalgebra::Point3<T>
    where T: Copy + ops::BitAnd<Output = T>
{
    fn mask(&self, value: T) -> Self {
        nalgebra::Point3::new(self.x & value, self.y & value, self.z & value)
    }
}

pub trait UpperBound {
    fn upper_bound(&self, other: &Self) -> Self;
}

impl<T> UpperBound for nalgebra::Point3<T>
    where T: Copy + Ord
{
    fn upper_bound(&self, other: &Self) -> Self {
        nalgebra::Point3::new(cmp::max(self.x, other.x),
                              cmp::max(self.y, other.y),
                              cmp::max(self.y, other.y))
    }
}

pub trait LowerBound {
    fn lower_bound(&self, other: &Self) -> Self;
}

impl<T> LowerBound for nalgebra::Point3<T>
    where T: Copy + Ord
{
    fn lower_bound(&self, other: &Self) -> Self {
        nalgebra::Point3::new(cmp::min(self.x, other.x),
                              cmp::min(self.y, other.y),
                              cmp::min(self.y, other.y))
    }
}

pub trait Interpolate<F> {
    fn lerp(&self, other: &Self, f: F) -> Self;
}

impl<T, F> Interpolate<F> for (T, T, T)
    where T: nalgebra::Cast<F> + Copy + Num,
          F: nalgebra::Cast<T> + Float
{
    fn lerp(&self, other: &Self, f: F) -> Self {
        (lerp(self.0, other.0, f), lerp(self.1, other.1, f), lerp(self.2, other.2, f))
    }
}

impl<T, F> Interpolate<F> for nalgebra::Point3<T>
    where T: nalgebra::Cast<F> + Copy + Num,
          F: nalgebra::Cast<T> + Float
{
    fn lerp(&self, other: &Self, f: F) -> Self {
        nalgebra::Point3::new(lerp(self.x, other.x, f),
                              lerp(self.y, other.y, f),
                              lerp(self.z, other.z, f))
    }
}

pub fn lerp<T, F>(a: T, b: T, f: F) -> T
    where T: nalgebra::Cast<F> + Copy + Num,
          F: nalgebra::Cast<T> + Float
{
    use nalgebra::{cast, clamp};

    let f = clamp(f, F::zero(), F::one());
    let af = cast::<T, F>(a) * (F::one() - f);
    let bf = cast::<T, F>(b) * f;
    cast::<F, T>(af + bf)
}

#[cfg(test)]
mod tests {
    use super::*;
}
