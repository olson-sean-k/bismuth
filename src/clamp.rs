use nalgebra;
use num::Num;
use std::cmp::Ordering;
use std::convert::From;
use std::marker::PhantomData;
use std::ops;

use math::Clamp;

pub trait ClampedRange<T> {
    fn max_value() -> T;
    fn min_value() -> T;
}

#[derive(Clone, Copy)]
pub struct Clamped<T, R>(T, PhantomData<R>)
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T>;

impl<T, R> Clamped<T, R>
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    pub fn new(value: T) -> Self {
        Clamped(
            nalgebra::clamp(value, R::min_value(), R::max_value()),
            PhantomData,
        )
    }

    pub fn max_value() -> Self {
        Clamped(R::max_value(), PhantomData)
    }

    pub fn max_inner_value() -> T {
        R::max_value()
    }

    pub fn min_value() -> Self {
        Clamped(R::min_value(), PhantomData)
    }

    pub fn min_inner_value() -> T {
        R::min_value()
    }

    pub fn to_inner(&self) -> T {
        self.0
    }
}

impl<T, R> Clamp<Clamped<T, R>> for Clamped<T, R>
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T> + Copy,
{
    fn clamp(&self, min: Self, max: Self) -> Self {
        nalgebra::clamp(*self, min, max)
    }
}

impl<T, R> From<T> for Clamped<T, R>
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T, R> Eq for Clamped<T, R>
where
    T: Copy + Eq + Num + PartialOrd,
    R: ClampedRange<T>,
{
}

impl<T, R> Ord for Clamped<T, R>
where
    T: Copy + Num + Ord + PartialOrd,
    R: ClampedRange<T>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, R> PartialEq for Clamped<T, R>
where
    T: Copy + Num + PartialEq + PartialOrd,
    R: ClampedRange<T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, R> PartialEq<T> for Clamped<T, R>
where
    T: Copy + Num + PartialEq + PartialOrd,
    R: ClampedRange<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T, R> PartialOrd for Clamped<T, R>
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T, R> PartialOrd<T> for Clamped<T, R>
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<T, R> ops::Add for Clamped<T, R>
where
    T: ops::Add<Output = T> + Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.0.add(other.0))
    }
}

impl<T, R> ops::Add<T> for Clamped<T, R>
where
    T: ops::Add<Output = T> + Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn add(self, other: T) -> Self::Output {
        Self::new(self.0.add(other))
    }
}

impl<T, R> ops::Deref for Clamped<T, R>
where
    T: Copy + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, R> ops::Div for Clamped<T, R>
where
    T: Copy + ops::Div<Output = T> + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        Self::new(self.0.div(other.0))
    }
}

impl<T, R> ops::Div<T> for Clamped<T, R>
where
    T: Copy + ops::Div<Output = T> + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn div(self, other: T) -> Self::Output {
        Self::new(self.0.div(other))
    }
}

impl<T, R> ops::Mul for Clamped<T, R>
where
    T: Copy + ops::Mul<Output = T> + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self::new(self.0.mul(other.0))
    }
}

impl<T, R> ops::Mul<T> for Clamped<T, R>
where
    T: Copy + ops::Mul<Output = T> + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Self::new(self.0.mul(other))
    }
}

impl<T, R> ops::Neg for Clamped<T, R>
where
    T: Copy + ops::Neg<Output = T> + Num + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(self.0.neg())
    }
}

impl<T, R> ops::Sub for Clamped<T, R>
where
    T: Copy + Num + ops::Sub<Output = T> + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.0.sub(other.0))
    }
}

impl<T, R> ops::Sub<T> for Clamped<T, R>
where
    T: Copy + Num + ops::Sub<Output = T> + PartialOrd,
    R: ClampedRange<T>,
{
    type Output = Self;

    fn sub(self, other: T) -> Self::Output {
        Self::new(self.0.sub(other))
    }
}
