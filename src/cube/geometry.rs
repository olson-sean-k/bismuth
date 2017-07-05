use clamp::{Clamped, ClampedRange};
use math::{Clamp, FPoint3, FromSpace, FScalar, UPoint3};
use super::space::Axis;

/// Defines the bounds for `Offset` values.
#[derive(Clone, Copy)]
pub struct OffsetRange;

impl ClampedRange<u8> for OffsetRange {
    fn max_value() -> u8 {
        0x0F
    }

    fn min_value() -> u8 {
        0x00
    }
}

/// Edge offset.
///
/// This is the deformation of an `Edge` in a particular direction as an
/// absolute coordinate. It is used to define the endpoints of an `Edge`.
///
/// `Offset` is clamped to the open range `[0, 15]`. This is the same range that
/// the endpoints of an `Edge` are allowed to span.
pub type Offset = Clamped<u8, OffsetRange>;

/// Edge of a cube's `Geometry`.
///
/// `Edge`s define the endpoints of a cube's points along a particular axis as
/// front and back. For each axis, there are four edges describing the
/// coordinates of all eight points along that axis. The front and back of an
/// `Edge` are described by an `Offset` and affect opposing faces of a cube.
///
/// The front and back of an `Edge` cannot cross, but may intersect. These
/// values are packed into the upper and lower 4-bit halves of an `u8`.
#[derive(Copy, Clone)]
pub struct Edge(u8);

impl Edge {
    /// Constructs a new `Edge` that spans the full width (no deformation).
    fn full() -> Self {
        Edge(Offset::max_inner_value())
    }

    /// Constructs a new `Edge` that converges at a given `Offset`.
    fn converged(offset: Offset) -> Self {
        let offset = offset.to_inner();
        Edge((offset << 4) | offset)
    }

    /// Sets the offset of the front of the `Edge`. If the front would cross the
    /// back, it will be clamped such that it intersects the back.
    pub fn set_front(&mut self, offset: Offset) {
        let offset = offset.clamp(Offset::min_value(), self.back()).to_inner();
        self.0 = (offset << 4) | self.back().to_inner();
    }

    /// Sets the `Offset` of the back of the `Edge`. If the back would cross the
    /// front, it will be clamped such that it intersects the front.
    pub fn set_back(&mut self, offset: Offset) {
        let offset = offset.clamp(self.front(), Offset::max_value()).to_inner();
        self.0 = (self.front().to_inner() << 4) | offset;
    }

    /// Gets the `Offset` of the front of the `Edge`.
    pub fn front(&self) -> Offset {
        Offset::from((self.0 & 0xF0) >> 4)
    }

    /// Gets the `Offset` of the back of the `Edge`.
    pub fn back(&self) -> Offset {
        Offset::from(self.0 & 0x0F)
    }

    /// Gets the length of the `Edge`.
    pub fn length(&self) -> Offset {
        self.back() - self.front()
    }

    fn front_unit_transform(&self) -> FScalar {
        let min = Offset::min_inner_value();
        let n = (self.front().to_inner() - min) as FScalar;
        let d = (Offset::max_inner_value() - min) as FScalar;
        n / d
    }

    fn back_unit_transform(&self) -> FScalar {
        let min = Offset::min_inner_value();
        let span = Offset::max_inner_value() - min;
        let n = -((span - (self.back().to_inner() - min)) as FScalar);
        let d = span as FScalar;
        n / d
    }
}

#[derive(Copy, Clone)]
pub struct Geometry([[Edge; 4]; 3]);

impl Geometry {
    pub fn full() -> Self {
        Geometry([[Edge::full(); 4]; 3])
    }

    pub fn empty() -> Self {
        Geometry([[Edge::converged(Offset::from(0)); 4]; 3])
    }

    pub fn edges(&self, axis: Axis) -> &[Edge; 4] {
        &self.0[axis as usize]
    }

    pub fn edges_mut(&mut self, axis: Axis) -> &mut [Edge; 4] {
        &mut self.0[axis as usize]
    }

    pub fn is_empty(&self) -> bool {
        self.0
            .iter()
            .any(|axis| axis.iter().all(|edge| edge.length() == 0))
    }

    pub fn map_unit_cube_point(&self, unit: &UPoint3) -> FPoint3 {
        let mut point = FPoint3::from_space(*unit);
        for axis in Axis::range() {
            let edge = &self.0[axis][index_at_axis(axis, unit)];
            point[axis] += if unit[axis] == 0 {
                edge.front_unit_transform()
            }
            else {
                edge.back_unit_transform()
            };
        }
        point
    }
}

/// Gets the index of an `Edge` in a face (a collection of four `Edge`s along a
/// particular axis).
fn index_at_axis(axis: usize, unit: &UPoint3) -> usize {
    let p = if axis == 0 { 1 } else { 0 };
    let q = if axis == 2 { 1 } else { 2 };
    (unit[p] | (unit[q] << 1)) as usize
}
