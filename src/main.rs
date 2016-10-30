extern crate nalgebra;

mod cube;
mod render;
mod resource;

use cube::*;

fn main() {
    let point = Point::new(1024, 197, 293);
    let mut tree = Tree::new(10);
    {
        let mut cube = tree.cursor_mut();
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        cube.subdivide().unwrap();
    }
    for cube in tree.cursor().iter().filter(|cube| cube.is_leaf()) {
        println!("{}W at {}",
                 cube.partition().width(),
                 cube.partition().origin());
    }
}
