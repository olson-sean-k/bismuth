extern crate nalgebra;

use super::space::*;

#[derive(Copy, Clone)]
pub struct Edge(u8);

pub const MIN_EDGE: u8 = 0;
pub const MAX_EDGE: u8 = 0x0F;

impl Edge {
    fn full() -> Self {
        Edge(MAX_EDGE)
    }

    fn converged(value: u8) -> Self {
        let value = nalgebra::clamp(value, MIN_EDGE, MAX_EDGE);
        Edge((value << 4) | value)
    }

    pub fn set_front(&mut self, value: u8) {
        let value = nalgebra::clamp(value, MIN_EDGE, self.back());
        self.0 = (value << 4) | self.back();
    }

    pub fn set_back(&mut self, value: u8) {
        let value = nalgebra::clamp(value, self.front(), MAX_EDGE);
        self.0 = (self.front() << 4) | value;
    }

    pub fn front(&self) -> u8 {
        (self.0 & 0xF0) >> 4
    }

    pub fn back(&self) -> u8 {
        self.0 & 0x0F
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

    pub fn axis_edges(&self, axis: Axis) -> &[Edge; 4] {
        &self.0[axis as usize]
    }

    pub fn axis_edges_mut(&mut self, axis: Axis) -> &mut [Edge; 4] {
        &mut self.0[axis as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
