#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra;
extern crate rand;

use gfx::format;
use gfx::Device;
use gfx::traits::FactoryExt;
use nalgebra::ToHomogeneous;

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

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 4]) -> Self {
        Vertex {
            position: position,
            color: color,
        }
    }
}

gfx_pipeline!{
    pipeline {
        vertex_buffer: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_transform",
        output: gfx::RenderTarget<gfx::format::Rgba8> = "f_target0",
    }
}

fn new_tree() -> cube::Tree {
    let point = cube::Point::new(0, 0, 0);
    let mut tree = cube::Tree::new(10);
    {
        let mut cube = tree.cursor_mut();
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        let mut cube = cube.subdivide().unwrap().resolve(&point, 0);
        cube.subdivide().unwrap();
    }
    tree
}

fn get_tree_vertex_data(tree: &cube::Tree) -> (Vec<Vertex>, Vec<u16>) {
    let CUBE_MESH_VERTEX: [nalgebra::Point3<f32>; 24] = [
        nalgebra::Point3::new(0.0, 0.0, 1.0),
        nalgebra::Point3::new(1.0, 0.0, 1.0),
        nalgebra::Point3::new(1.0, 1.0, 1.0),
        nalgebra::Point3::new(0.0, 1.0, 1.0),

        nalgebra::Point3::new(0.0, 1.0, 0.0),
        nalgebra::Point3::new(1.0, 1.0, 0.0),
        nalgebra::Point3::new(1.0, 0.0, 0.0),
        nalgebra::Point3::new(0.0, 0.0, 0.0),

        nalgebra::Point3::new(1.0, 0.0, 0.0),
        nalgebra::Point3::new(1.0, 1.0, 0.0),
        nalgebra::Point3::new(1.0, 1.0, 1.0),
        nalgebra::Point3::new(1.0, 0.0, 1.0),

        nalgebra::Point3::new(0.0, 0.0, 1.0),
        nalgebra::Point3::new(0.0, 1.0, 1.0),
        nalgebra::Point3::new(0.0, 1.0, 0.0),
        nalgebra::Point3::new(0.0, 0.0, 0.0),

        nalgebra::Point3::new(1.0, 1.0, 0.0),
        nalgebra::Point3::new(0.0, 1.0, 0.0),
        nalgebra::Point3::new(0.0, 1.0, 1.0),
        nalgebra::Point3::new(1.0, 1.0, 1.0),

        nalgebra::Point3::new(1.0, 0.0, 1.0),
        nalgebra::Point3::new(0.0, 0.0, 1.0),
        nalgebra::Point3::new(0.0, 0.0, 0.0),
        nalgebra::Point3::new(1.0, 0.0, 0.0),
    ];
    const CUBE_MESH_INDEX: [u16; 36] = [
        0,  1,  2,  2,  3,  0,
        4,  5,  6,  6,  7,  4,
        8,  9,  10, 10, 11, 8,
        12, 13, 14, 14, 15, 12,
        16, 17, 18, 18, 19, 16,
        20, 21, 22, 22, 23, 20
    ];

    let mut vertex_data = Vec::new();
    let mut index_data: Vec<u16> = Vec::new();
    for (i, cube) in tree.cursor().iter().filter(|cube| cube.is_leaf()).enumerate() {
        let width = cube.partition().width();
        let origin = cube.partition().origin();
        let origin = nalgebra::Vector3::<f32>::new(origin.x as f32,
                                                   origin.y as f32,
                                                   origin.z as f32);
        let color = [rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>(), 0.1];
        vertex_data.extend(CUBE_MESH_VERTEX.iter()
            .map(|point| (point * cube::exp(width) as f32) + origin)
            .map(|point| Vertex::new(*point.as_ref(), color)));
        index_data.extend(CUBE_MESH_INDEX.iter().map(|j| ((24 * i as u16) + *j)));
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

    let tree = new_tree();
    let look = cube::exp(tree.partition().width() - 1) as f32;
    let view = nalgebra::Isometry3::look_at_rh(
        &nalgebra::Point3::<f32>::new(look * 0.25, -look, -look * 2.0),
        &nalgebra::Point3::<f32>::new(look, look, look),
        &nalgebra::Vector3::<f32>::new(0.0, 0.0, 1.0)).to_homogeneous();
    let projection = nalgebra::PerspectiveMatrix3::new(4.0 / 3.0, 1.0, -1.0, 1.0).to_matrix();
    let transform = projection * view;
    let (vertex_buffer, slice) = {
        let (vertex_data, index_data) = get_tree_vertex_data(&tree);
        factory.create_vertex_buffer_with_slice(vertex_data.as_slice(), index_data.as_slice())
    };
    let data = pipeline::Data {
        vertex_buffer: vertex_buffer,
        transform: *transform.as_ref(),
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
