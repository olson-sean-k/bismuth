extern crate nalgebra;

use clamp::{Clamped, ClampedRange};
use math::{Clamp, FPoint3, FromSpace, UPoint3};
use super::space::Axis;

pub struct OffsetRange;

impl ClampedRange<u8> for OffsetRange {
    fn max_value() -> u8 {
        0x0F
    }

    fn min_value() -> u8 {
        0x00
    }
}

pub type Offset = Clamped<u8, OffsetRange>;

#[derive(Copy, Clone)]
pub struct Edge(u8);

impl Edge {
    fn full() -> Self {
        Edge(Offset::max_inner_value())
    }

    fn converged(offset: Offset) -> Self {
        let offset = offset.to_inner();
        Edge((offset << 4) | offset)
    }

    pub fn set_front(&mut self, offset: Offset) {
        let offset = offset.clamp(Offset::min_value(), self.back()).to_inner();
        self.0 = (offset << 4) | self.back().to_inner();
    }

    pub fn set_back(&mut self, offset: Offset) {
        let offset = offset.clamp(self.front(), Offset::max_value()).to_inner();
        self.0 = (self.front().to_inner() << 4) | offset;
    }

    pub fn front(&self) -> Offset {
        Offset::from((self.0 & 0xF0) >> 4)
    }

    pub fn back(&self) -> Offset {
        Offset::from(self.0 & 0x0F)
    }

    pub fn length(&self) -> Offset {
        self.back() - self.front()
    }

    fn front_unit_transform(&self) -> f32 {
        let min = Offset::min_inner_value();
        ((self.front().to_inner() - min) as f32) / ((Offset::max_inner_value() - min) as f32)
    }

    fn back_unit_transform(&self) -> f32 {
        let len = Offset::max_inner_value() - Offset::min_inner_value();
        -((len - (self.back().to_inner() - Offset::min_inner_value())) as f32) / (len as f32)
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
        self.0.iter().any(|axis| axis.iter().all(|edge| edge.length() == 0))
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

fn index_at_axis(axis: usize, unit: &UPoint3) -> usize {
    let p = if axis == 0 { 1 } else { 0 };
    let q = if axis == 2 { 1 } else { 2 };
    (unit[p] | (unit[q] << 1)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
}
