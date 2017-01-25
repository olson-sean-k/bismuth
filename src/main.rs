extern crate bismuth;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate nalgebra;

use bismuth::cube::{Axis, Cursor, LogWidth, Offset, Root};
use bismuth::render;
use bismuth::prelude::*;
use gfx::{format, Device};
use gfx::traits::FactoryExt;
use nalgebra::Origin;

const CLEAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn new_root() -> Root {
    let width = LogWidth::max_value();
    let cursor = Cursor::span_from_point(&UPoint3::origin(), &UVector3::new(7, 1, 7), width - 3);
    let mut root = Root::new(width);
    for mut cube in root.to_cube_mut().subdivide_to_cursor(&cursor).iter_mut() {
        for mut cube in cube.iter_mut() {
            if let Some(leaf) = cube.try_as_leaf_mut() {
                for axis in Axis::range() {
                    for edge in leaf.geometry.edges_mut(axis.into()) {
                        edge.set_front(Offset::from(2));
                        edge.set_back(Offset::from(14));
                    }
                }
            }
        }
    }
    root
}

fn main() {
    let (window, mut device, mut factory, color, _) =
        gfx_window_glutin::init::<format::Rgba8,
                                  format::DepthStencil>(glutin::WindowBuilder::new()
            .with_title("Bismuth")
            .with_dimensions(640, 480)
            .with_vsync());
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let state = factory.create_pipeline_simple(include_bytes!("shader/cube.v.glsl"),
                                               include_bytes!("shader/cube.f.glsl"),
                                               render::pipeline::new())
        .unwrap();

    let root = new_root();
    let transform = {
        let midpoint: FPoint3 = root.partition().midpoint().into_space();
        let camera = FPoint3::new(midpoint.x * 0.25, -midpoint.y, -midpoint.z * 2.0);
        let view = render::look_at_cube(&root, &camera);
        let projection = render::projection_from_window(&window);
        projection * view
    };
    let (vertex_buffer, slice) = render::vertex_buffer_from_cube(&root.to_cube(), &mut factory);
    let data = render::pipeline::Data {
        vertex_buffer: vertex_buffer,
        transform: *transform.as_ref(),
        output: color,
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
