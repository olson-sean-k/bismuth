#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate num;
extern crate rand;

use gfx::format;
use gfx::Device;
use gfx::traits::FactoryExt;

mod cube;
mod math;
mod render;
mod resource;

use cube::ComputedCube;
pub use math::*;

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn new_tree() -> cube::Tree {
    let point = cube::Point3::new(0, 0, 0);
    let mut tree = cube::Tree::new(10);
    {
        let mut cube = tree.cursor_mut();
        let mut cube = cube.subdivide().unwrap().at_point(&point, 0);
        let mut cube = cube.subdivide().unwrap().at_point(&point, 0);
        cube.subdivide().unwrap();
    }
    tree
}

fn main() {
    let (window, mut device, mut factory, surface, _) =
        gfx_window_glutin::init::<format::Rgba8,
                                  format::DepthStencil>(glutin::WindowBuilder::new()
            .with_title("Bismuth")
            .with_dimensions(640, 480)
            .with_vsync());
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let state = factory.create_pipeline_simple(include_bytes!("shader/cube.glslv"),
                                include_bytes!("shader/cube.glslf"),
                                render::pipeline::new())
        .unwrap();

    let tree = new_tree();
    let transform = {
        let midpoint: render::Point3 = tree.partition().midpoint().into_domain();
        let camera = render::Point3::new(midpoint.x * 0.25, -midpoint.y, -midpoint.z * 2.0);
        let view = render::look_at_cube(&tree, &camera);
        let projection = render::projection_from_window(&window);
        projection * view
    };
    let (vertex_buffer, slice) = render::vertex_buffer_from_cube(&tree.cursor(), &mut factory);
    let data = render::pipeline::Data {
        vertex_buffer: vertex_buffer,
        transform: *transform.as_ref(),
        output: surface,
    };

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {}
            }
        }

        encoder.clear(&data.output, CLEAR_COLOR);
        encoder.draw(&slice, &state, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
