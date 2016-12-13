extern crate nalgebra;

use std::ops;

pub type DiscreteScalar = u32;
pub type RealScalar = f32;

pub trait FromSpace<T> {
    fn from_space(value: T) -> Self;
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

pub trait Mask<T>
    where T: ops::BitAnd<Output = T>
{
    fn mask(&self, value: T) -> Self;
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

impl<T> Clamp<T> for nalgebra::Point3<T>
    where T: Copy,
          T: PartialOrd
{
    fn clamp(&self, min: T, max: T) -> Self {
        nalgebra::Point3::new(nalgebra::clamp(self.x, min, max),
                              nalgebra::clamp(self.y, min, max),
                              nalgebra::clamp(self.z, min, max))
    }
}

impl<T> Mask<T> for nalgebra::Point3<T>
    where T: Copy,
          T: ops::BitAnd<Output = T>
{
    fn mask(&self, value: T) -> Self {
        nalgebra::Point3::new(self.x & value, self.y & value, self.z & value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
