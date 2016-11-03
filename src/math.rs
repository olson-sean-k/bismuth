extern crate nalgebra;
extern crate num;

use num::cast;
use std::ops;

pub trait FromDomain<T> {
    fn from_domain(value: T) -> Self;
}

pub trait IntoDomain<T> {
    fn into_domain(self) -> T;
}

impl<T, U> IntoDomain<U> for T
    where U: FromDomain<T>
{
    fn into_domain(self) -> U {
        U::from_domain(self)
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

impl<T, U> FromDomain<nalgebra::Point3<U>> for nalgebra::Point3<T>
    where T: cast::NumCast,
          U: num::ToPrimitive
{
    fn from_domain(point: nalgebra::Point3<U>) -> Self {
        nalgebra::Point3::new(T::from(point.x).unwrap(),
                              T::from(point.y).unwrap(),
                              T::from(point.z).unwrap())
    }
}

impl<T, U> FromDomain<nalgebra::Vector3<U>> for nalgebra::Vector3<T>
    where T: cast::NumCast,
          U: num::ToPrimitive
{
    fn from_domain(vector: nalgebra::Vector3<U>) -> Self {
        nalgebra::Vector3::new(T::from(vector.x).unwrap(),
                               T::from(vector.y).unwrap(),
                               T::from(vector.z).unwrap())
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
