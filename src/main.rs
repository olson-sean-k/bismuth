extern crate bismuth;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate num;

use bismuth::{cube, edit, render};
use bismuth::prelude::*;
use gfx::{format, Device};
use gfx::traits::FactoryExt;

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn new_root() -> cube::Root {
    let width = cube::MAX_WIDTH;
    let cursor = edit::Selection::at_point(&cube::Point3::new(0, 0, 0), width - 3)
        .span(&cube::Vector3::new(7, 1, 7))
        .to_cursor();
    let mut root = cube::Root::new(width);
    root.to_cube_mut().subdivide_to_cursor(&cursor);
    root
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

    let root = new_root();
    let transform = {
        let midpoint: render::Point3 = root.partition().midpoint().into_domain();
        let camera = render::Point3::new(midpoint.x * 0.25, -midpoint.y, -midpoint.z * 2.0);
        let view = render::look_at_cube(&root, &camera);
        let projection = render::projection_from_window(&window);
        projection * view
    };
    let (vertex_buffer, slice) = render::vertex_buffer_from_cube(&root.to_cube(), &mut factory);
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
