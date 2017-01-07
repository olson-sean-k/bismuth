extern crate nalgebra;

use math::{FPoint3, FromSpace, UPoint3};
use super::space::Axis;

pub type Offset = u8;

pub const MIN_OFFSET: Offset = 0;
pub const MAX_OFFSET: Offset = 0x0F;

#[derive(Copy, Clone)]
pub struct Edge(u8);

impl Edge {
    fn full() -> Self {
        Edge(MAX_OFFSET)
    }

    fn converged(offset: Offset) -> Self {
        let offset = nalgebra::clamp(offset, MIN_OFFSET, MAX_OFFSET);
        Edge((offset << 4) | offset)
    }

    pub fn set_front(&mut self, offset: Offset) {
        let offset = nalgebra::clamp(offset, MIN_OFFSET, self.back());
        self.0 = (offset << 4) | self.back();
    }

    pub fn set_back(&mut self, offset: Offset) {
        let offset = nalgebra::clamp(offset, self.front(), MAX_OFFSET);
        self.0 = (self.front() << 4) | offset;
    }

    pub fn front(&self) -> Offset {
        (self.0 & 0xF0) >> 4
    }

    pub fn back(&self) -> Offset {
        self.0 & 0x0F
    }

    pub fn length(&self) -> Offset {
        self.back() - self.front()
    }

    fn front_unit_transform(&self) -> f32 {
        ((self.front() - MIN_OFFSET) as f32) / ((MAX_OFFSET - MIN_OFFSET) as f32)
    }

    fn back_unit_transform(&self) -> f32 {
        let range = MAX_OFFSET - MIN_OFFSET;
        -((range - (self.back() - MIN_OFFSET)) as f32) / (range as f32)
    }
}

#[derive(Copy, Clone)]
pub struct Geometry([[Edge; 4]; 3]);

impl Geometry {
    pub fn full() -> Self {
        Geometry([[Edge::full(); 4]; 3])
    }

    pub fn empty() -> Self {
        Geometry([[Edge::converged(0); 4]; 3])
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
