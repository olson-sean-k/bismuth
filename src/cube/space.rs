use alga::general::SupersetOf;
use nalgebra::{Point3, Scalar, Unit};
use num::{Bounded, One, Zero}; // TODO: `use ::std::num::{One, Zero};`.
use std::ops::Range;

use clamp::{Clamped, ClampedRange};
use math::{self, FRay3, FPoint3, FromSpace, FScalar, FVector3, LowerBound, Mask, UPoint3,
           UpperBound, UScalar, UVector3};

/// Defines the bounds for `LogWidth` values.
#[derive(Clone, Copy)]
pub struct LogWidthRange;

impl ClampedRange<u8> for LogWidthRange {
    fn max_value() -> u8 {
        31
    }

    fn min_value() -> u8 {
        4
    }
}

/// Logarithmic width. Most uses of the term "width" refer to `LogWidth`.
///
/// This is the base-2 logarithmic width of a space in a tree, such that a width
/// `x` exponentiates as `2ˣ`. Exponentiation is performed as a simple bit
/// shift.
///
/// Being logarithmic, if a cube has a width `x`, then its immediate
/// subdivisions have a width `x - 1` and its parent a width `x + 1`.
///
/// `LogWidth` is clamped to the open range `[4, 31]`.
pub type LogWidth = Clamped<u8, LogWidthRange>;

impl LogWidth {
    /// Reference width of `Cube`s.
    pub fn unit() -> Self {
        LogWidth::new(8)
    }

    /// Exponentiates a `LogWidth` into a `UScalar`.
    pub fn exp(&self) -> UScalar {
        UScalar::one() << self.to_inner()
    }
}

/// An axis in the tree space.
///
/// Defines and orders the axes in the tree space.
#[derive(Clone, Copy)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    /// Gets a `Range` over axes as `usize`s.
    pub fn range() -> Range<usize> {
        (Axis::X as usize)..(Axis::Z as usize + 1)
    }

    pub fn to_vector(&self) -> FVector3 {
        match *self {
            Axis::X => FVector3::x(),
            Axis::Y => FVector3::y(),
            Axis::Z => FVector3::z(),
        }
    }
}

impl From<usize> for Axis {
    fn from(index: usize) -> Self {
        match index {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!(), // TODO: Use `TryFrom`.
        }
    }
}

/// A direction along an `Axis`.
pub enum Direction {
    Positive,
    Negative,
}

/// An orientation in the tree space.
///
/// This is akin to choosing a face of a cube in a tree and orienting that face
/// based on the axis and direction in which it deforms. For example, `Left` is
/// a face along the `X` axis that deforms in the `Positive` direction.
pub enum Orientation {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

impl Orientation {
    /// Gets the `Axis` associated with the `Orientation`.
    pub fn axis(&self) -> Axis {
        match *self {
            Orientation::Left | Orientation::Right => Axis::X,
            Orientation::Top | Orientation::Bottom => Axis::Y,
            Orientation::Front | Orientation::Back => Axis::Z,
        }
    }

    /// Gets the `Direction` associated with the `Orientation`.
    pub fn direction(&self) -> Direction {
        match *self {
            Orientation::Left | Orientation::Bottom | Orientation::Back => Direction::Positive,
            Orientation::Right | Orientation::Top | Orientation::Front => Direction::Negative,
        }
    }
}

pub trait PointNormal {
    fn normal(&self, point: &FPoint3) -> FVector3;
}

/// Shape or primitive that can test for intersection with another shape or
/// primitive.
pub trait Intersects<T> {
    /// Tests for intersection.
    fn intersects(&self, other: &T) -> bool;
}

/// Details of intersection with an `FRay3`.
pub struct RayIntersection {
    /// The minimum time of impact.
    pub distance: FScalar,
    /// The minimum point of intersection.
    pub point: FPoint3,
    /// The normal at the point of intersection.
    pub normal: Unit<FVector3>,
}

impl RayIntersection {
    /// Creates a new `RayIntersection`. Normalizes `normal`.
    fn new(distance: FScalar, point: FPoint3, normal: FVector3) -> Self {
        RayIntersection {
            distance: distance,
            point: point,
            normal: Unit::new_normalize(normal),
        }
    }
}

/// Shape or primitive that can test for intersection with an `FRay3`, omitting
/// some details about the intersection.
pub trait PartialRayCast {
    /// Determines if an `FRay3` intersects the shape or primitive. Returns the
    /// minimum and maximum time of impact (distance), respectively.
    fn partial_ray_intersection(&self, ray: &FRay3) -> Option<(FScalar, FScalar)>;
}

/// Shape or primitive that can test for intersection with an `FRay3`.
pub trait RayCast: PartialRayCast {
    /// Determines if an `FRay3` intersects the shape or primitive and returns
    /// details about the intersection.
    fn ray_intersection(&self, ray: &FRay3) -> Option<RayIntersection>;
}

impl<T> Intersects<FRay3> for T
where
    T: PartialRayCast,
{
    fn intersects(&self, ray: &FRay3) -> bool {
        self.partial_ray_intersection(ray).is_some()
    }
}

impl<T> RayCast for T
where
    T: PartialRayCast + PointNormal,
{
    fn ray_intersection(&self, ray: &FRay3) -> Option<RayIntersection> {
        self.partial_ray_intersection(ray).map(|(distance, _)| {
            let point = ray.origin + (*ray.direction * distance);
            RayIntersection::new(distance, point, self.normal(&point))
        })
    }
}

/// Axis-aligned bounding box.
///
/// `AABB`s are represented as an origin and extent.
pub struct AABB {
    pub origin: UPoint3,
    pub extent: UVector3,
}

impl AABB {
    /// Constructs a new `AABB` at the given point in space with the given
    /// extent.
    pub fn new(origin: UPoint3, extent: UVector3) -> Self {
        AABB {
            origin: origin,
            extent: extent,
        }
    }

    /// Constructs the union of two `AABB`s.
    ///
    /// The union is the cuboid formed from the lower and upper bounds of the
    /// `AABB`s.
    pub fn union(&self, other: &Self) -> Self {
        let start = self.origin.lower_bound(&other.origin);
        let end = self.endpoint().upper_bound(&other.endpoint());
        AABB::new(start, end - start)
    }

    /// Gets the midpoint (center) of the `AABB`.
    pub fn midpoint(&self) -> UPoint3 {
        self.origin + (self.extent / 2)
    }

    /// Gets the absolute endpoint of the `AABB`.
    pub fn endpoint(&self) -> UPoint3 {
        self.origin + self.extent
    }
}

impl Intersects<AABB> for AABB {
    /// Determines if two `AABB`s intersect.
    fn intersects(&self, aabb: &AABB) -> bool {
        for axis in Axis::range() {
            if (self.origin[axis] + self.extent[axis]) < aabb.origin[axis] {
                return false;
            }
            if self.origin[axis] > (aabb.origin[axis] + aabb.extent[axis]) {
                return false;
            }
        }
        true
    }
}

impl<T> Intersects<Point3<T>> for AABB
where
    T: PartialOrd + Scalar + SupersetOf<UScalar>,
{
    /// Determines if a point intersects an `AABB`.
    fn intersects(&self, point: &Point3<T>) -> bool {
        use nalgebra::convert;

        for axis in Axis::range() {
            if convert::<UScalar, T>(self.origin[axis] + self.extent[axis]) < point[axis] {
                return false;
            }
            if convert::<UScalar, T>(self.origin[axis]) > point[axis] {
                return false;
            }
        }
        true
    }
}

impl PartialRayCast for AABB {
    /// Determines if an `FRay3` intersects an `AABB`. Returns the minimum and
    /// maximum times of impact as a tuple, respectively.
    fn partial_ray_intersection(&self, ray: &FRay3) -> Option<(FScalar, FScalar)> {
        let mut min = FVector3::zero();
        let mut max = FVector3::zero();
        for axis in Axis::range() {
            let lower = self.origin[axis] as FScalar;
            let upper = lower + self.extent[axis] as FScalar;
            let origin = ray.origin[axis];
            let direction = ray.direction[axis];

            let (lower, upper) =
                math::ordered_pair((lower - origin) / direction, (upper - origin) / direction);
            min[axis] = lower;
            max[axis] = upper;
        }

        let min = math::partial_max(math::partial_max(min.x, min.y), min.z);
        let max = math::partial_min(math::partial_min(max.x, max.y), max.z);
        if max < 0.0 || min > max {
            None
        }
        else {
            Some((min, max))
        }
    }
}

impl PointNormal for AABB {
    /// Gets the normal at a point along or near the surface of the `AABB`.
    fn normal(&self, point: &FPoint3) -> FVector3 {
        let point = point - FPoint3::from_space(self.midpoint());
        let mut min_distance = FScalar::max_value();
        let mut normal = FVector3::zero();
        for axis in Axis::range() {
            let distance = (self.extent[axis] as FScalar - point[axis].abs()).abs();
            if distance < min_distance {
                min_distance = distance;
                normal = Axis::from(axis).to_vector() * point[axis].signum();
            }
        }
        normal
    }
}

/// A cubic spatial partition. `Partition`s are represented as an origin and a
/// width.
///
/// `Partition`s are associated with every `Cube` in a tree.
#[derive(Clone, Copy)]
pub struct Partition {
    origin: UPoint3,
    width: LogWidth,
}

impl Partition {
    /// Constructs a new `Partition` at the given point in space with the given
    /// width.
    pub fn at_point(point: &UPoint3, width: LogWidth) -> Self {
        Partition {
            origin: point.mask(!UScalar::zero() << width.to_inner()),
            width: width,
        }
    }

    /// Constructs the sub-`Partition` at the given index. Returns `None` if
    /// there is no such `Partition`, because the minimum width has been
    /// exceeded.
    ///
    /// # Panics
    ///
    /// Panics if `index` is not within the range [0, 8).
    pub fn at_index(&self, index: usize) -> Option<Self> {
        if self.is_min_width() {
            None
        }
        else {
            let width = self.width - 1;
            Some(Partition {
                origin: self.origin + vector_at_index(index, width),
                width: width,
            })
        }
    }

    /// Gets the origin of the `Partition`.
    pub fn origin(&self) -> &UPoint3 {
        &self.origin
    }

    /// Gets the width of the `Partition`.
    pub fn width(&self) -> LogWidth {
        self.width
    }

    /// Gets the midpoint of the `Partition`.
    pub fn midpoint(&self) -> UPoint3 {
        let m = (self.width - 1).exp();
        self.origin + UVector3::new(m, m, m)
    }

    pub fn extent(&self) -> UVector3 {
        (UVector3::new(1, 1, 1) * self.width.exp()) - UVector3::new(1, 1, 1)
    }

    /// Gets the `AABB` of the `Partition`.
    pub fn aabb(&self) -> AABB {
        AABB::new(self.origin, self.extent())
    }

    /// Returns `true` if the `Partition` has the minimum possible width.
    pub fn is_min_width(&self) -> bool {
        self.width == LogWidth::min_value()
    }
}

/// A spatial (cubic) element in a tree.
pub trait Spatial {
    /// Gets the `Partition` of the `Spatial`.
    fn partition(&self) -> &Partition;

    /// Gets the depth of the `Spatial` in the tree.
    fn depth(&self) -> u8;

    /// Gets the `AABB` of the `Spatial`.
    fn aabb(&self) -> AABB {
        self.partition().aabb()
    }
}

/// Gets the subdivision index in a tree for a given point at the given width.
#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn index_at_point(point: &UPoint3, width: LogWidth) -> usize {
    let width = width.to_inner();
    (( (point.x >> width) & UScalar::one()      ) |
     (((point.y >> width) & UScalar::one()) << 1) |
     (((point.z >> width) & UScalar::one()) << 2)) as usize
}

/// Gets a vector to the origin of a subdivision in a tree at a given index and
/// width.
#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn vector_at_index(index: usize, width: LogWidth) -> UVector3 {
    assert!(index < 8);
    let index = index as UScalar;
    let width = width.exp();
    UVector3::new(
        ( index       & UScalar::one()) * width,
        ((index >> 1) & UScalar::one()) * width,
        ((index >> 2) & UScalar::one()) * width,
    )
}
