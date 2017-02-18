extern crate bismuth;
extern crate glutin;
extern crate nalgebra;

use bismuth::cube::{Axis, Cursor, LogWidth, Offset, Root, Spatial};
use bismuth::math::{FPoint3, IntoSpace, Matrix4Ext, UPoint3, UVector3};
use bismuth::render::{AspectRatio, Camera, Context, Mesh, Projection, Transform};
use glutin::{Event, VirtualKeyCode, WindowBuilder};

fn new_root() -> Root {
    let width = LogWidth::max_value();
    let cursor = Cursor::at_point_with_span(&UPoint3::origin(), width - 3, &UVector3::new(7, 1, 7));
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

fn new_camera<W, C>(window: &W, cube: &C) -> Camera
    where W: AspectRatio,
          C: Spatial
{
    let midpoint: FPoint3 = cube.partition().midpoint().into_space();
    let mut camera = Camera::new(window, &Projection::default());
    camera.look_at(&FPoint3::new(midpoint.x * 0.25, -midpoint.y, -midpoint.z * 2.0),
                   &midpoint);
    camera
}

fn main() {
    let mut context = Context::from_glutin_window(WindowBuilder::new()
        .with_title("Bismuth")
        .with_dimensions(640, 480)
        .with_vsync()
        .build()
        .unwrap());
    let root = new_root();
    let mesh = root.to_cube().mesh_buffer();
    let camera = new_camera(&context.window, &root);
    let mut transform = Transform::default();
    'main: loop {
        transform.camera = camera.transform().to_array();
        for event in context.window.poll_events() {
            match event {
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) | Event::Closed => {
                    break 'main;
                },
                _ => {},
            }
        }
        context.clear();
        context.set_transform(&transform).unwrap();
        context.draw_mesh_buffer(&mesh);
        context.flush().unwrap();
    }
}
