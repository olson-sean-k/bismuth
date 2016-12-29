extern crate nalgebra;

use math::{FPoint3, FromSpace, UPoint3};
use render::Index;
use super::space::Axis;

pub type Offset = u8;

pub const MIN_EDGE_OFFSET: Offset = 0;
pub const MAX_EDGE_OFFSET: Offset = 0x0F;

// TODO: Replace with mesh generation and the rendering module.
lazy_static! {
    pub static ref UNIT_CUBE_POINTS: [UPoint3; 8] = [
        // Back.
        UPoint3::new(0, 0, 1), // 0
        UPoint3::new(1, 0, 1), // 1
        UPoint3::new(1, 1, 1), // 2
        UPoint3::new(0, 1, 1), // 3
        // Front.
        UPoint3::new(0, 1, 0), // 4
        UPoint3::new(1, 1, 0), // 5
        UPoint3::new(1, 0, 0), // 6
        UPoint3::new(0, 0, 0), // 7
    ];
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub static ref UNIT_CUBE_INDECES: [Index; 36] = [
        0, 1, 2, 2, 3, 0,
        4, 5, 6, 6, 7, 4,
        6, 5, 2, 2, 1, 6,
        0, 3, 4, 4, 7, 0,
        5, 4, 3, 3, 2, 5,
        1, 0, 7, 7, 6, 1,
    ];
}

#[derive(Copy, Clone)]
pub struct Edge(u8);

impl Edge {
    fn full() -> Self {
        Edge(MAX_EDGE_OFFSET)
    }

    fn converged(offset: Offset) -> Self {
        let offset = nalgebra::clamp(offset, MIN_EDGE_OFFSET, MAX_EDGE_OFFSET);
        Edge((offset << 4) | offset)
    }

    pub fn set_front(&mut self, offset: Offset) {
        let offset = nalgebra::clamp(offset, MIN_EDGE_OFFSET, self.back());
        self.0 = (offset << 4) | self.back();
    }

    pub fn set_back(&mut self, offset: Offset) {
        let offset = nalgebra::clamp(offset, self.front(), MAX_EDGE_OFFSET);
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
        ((self.front() - MIN_EDGE_OFFSET) as f32) / ((MAX_EDGE_OFFSET - MIN_EDGE_OFFSET) as f32)
    }

    fn back_unit_transform(&self) -> f32 {
        let range = MAX_EDGE_OFFSET - MIN_EDGE_OFFSET;
        -((range - (self.back() - MIN_EDGE_OFFSET)) as f32) / (range as f32)
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

    // TODO: Replace with mesh generation and the rendering module.
    pub fn points(&self) -> Vec<FPoint3> {
        UNIT_CUBE_POINTS.iter()
            .map(|unit| {
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
            })
            .collect()
    }
}

// TODO: Replace with mesh generation and the rendering module.
fn index_at_axis(axis: usize, unit: &UPoint3) -> usize {
    let p = if axis == 0 { 1 } else { 0 };
    let q = if axis == 2 { 1 } else { 2 };
    (unit[p] | (unit[q] << 1)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
}
