#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra;

use gfx::format;
use gfx::Device;
use gfx::traits::FactoryExt;

mod cube;
mod render;
mod resource;

use cube::Traversal;

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

gfx_vertex_struct!{
    Vertex {
        position: [f32; 3] = "a_position",
        color: [f32; 4] = "a_color",
    }
}

gfx_pipeline!{
    pipeline {
        vertex_buffer: gfx::VertexBuffer<Vertex> = (),
        output: gfx::RenderTarget<gfx::format::Rgba8> = "f_target0",
    }
}

fn new_tree() -> cube::Tree {
    let point = cube::Point::new(348, 256, 724);
    let mut tree = cube::Tree::new(10);
    {
        let mut cube = tree.cursor_mut();
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        cube.subdivide().unwrap();
    }
    tree
}

fn get_tree_vertex_data(tree: &cube::Tree) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertex_data = Vec::new();
    let mut index_data = Vec::new();
    for cube in tree.cursor().iter().filter(|cube| cube.is_leaf()) {
        // TODO: Generate vertex and index data.
    }
    (vertex_data, index_data)
}

fn main() {
    let (window, mut device, mut factory, surface, _) = gfx_window_glutin::init::<format::Rgba8, format::DepthStencil>(
        glutin::WindowBuilder::new()
            .with_title("Bismuth")
            .with_dimensions(640, 480)
            .with_vsync());
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let state = factory.create_pipeline_simple(
        include_bytes!("shader/cube.glslv"),
        include_bytes!("shader/cube.glslf"),
        pipeline::new()).unwrap();

    let (vertex_buffer, slice) = {
        let tree = new_tree();
        let (vertex_data, index_data) = get_tree_vertex_data(&tree);
        factory.create_vertex_buffer_with_slice(vertex_data.as_slice(),
                                                index_data.as_slice())
    };
    let data = pipeline::Data {
        vertex_buffer: vertex_buffer,
        output: surface,
    };

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        encoder.clear(&data.output, CLEAR_COLOR);
        encoder.draw(&slice, &state, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
