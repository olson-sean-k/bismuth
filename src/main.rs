extern crate bismuth;
extern crate glutin;
extern crate nalgebra;

use bismuth::cube::{Cursor, Geometry, LogWidth, Root, Spatial};
use bismuth::event::{ElementState, Event, MouseButton, Reactor};
use bismuth::framework::{Application, Harness};
use bismuth::input::{ElementTransition, Mouse};
use bismuth::math::{FMatrix4, FPoint3, FScalar, IntoSpace, UPoint3, UVector3};
use bismuth::render::{AspectRatio, Camera, Context, MeshBuffer, MetaContext, Projection,
                      ToMeshBuffer, Transform};
use glutin::WindowBuilder;

struct Bismuth {
    root: Root,
    mesh: MeshBuffer,
    camera: Camera,
    mouse: ElementTransition<MouseButton, Mouse>,
}

impl<C> Application<C> for Bismuth
    where C: MetaContext
{
    fn start(context: &mut Context<C>) -> Self {
        let root = new_root(LogWidth::new(8));
        let mesh = root.to_cube().to_mesh_buffer();
        let camera = new_camera(&context.window, &root);
        Bismuth {
            root: root,
            mesh: mesh,
            camera: camera,
            mouse: ElementTransition::new(Mouse::new()),
        }
    }

    fn update(&mut self, context: &mut Context<C>) {
        let mut dirty = false;
        if let Some(ElementState::Pressed) = self.mouse.transition(MouseButton::Left) {
            let ray = self.camera.cast_ray(&context.window, self.mouse.position());
            let mut cube = self.root.to_cube_mut();
            if let Some((_, mut cube)) = cube.at_ray_mut(&ray, LogWidth::min_value()) {
                if let Some(leaf) = cube.as_leaf_mut() {
                    leaf.geometry = Geometry::empty();
                    dirty = true;
                }
            }
        }
        if dirty {
            self.mesh = self.root.to_cube().to_mesh_buffer();
        }
        self.mouse.snapshot();
    }

    fn draw(&mut self, context: &mut Context<C>) {
        context.set_transform(&Transform::new(&self.camera.transform(),
                                              &FMatrix4::identity())).unwrap();
        context.draw_mesh_buffer(&self.mesh);
    }

    fn stop(self) {}
}

impl Reactor for Bismuth {
    fn react(&mut self, event: &Event) {
        self.camera.react(event);
        self.mouse.react(event);
    }
}

fn new_root(width: LogWidth) -> Root {
    let cursor = Cursor::at_point_with_span(&UPoint3::origin(), width - 3, &UVector3::new(7, 1, 7));
    let mut root = Root::new(width);
    root.to_cube_mut().subdivide_to_cursor(&cursor);
    root
}

fn new_camera<W, C>(window: &W, cube: &C) -> Camera
    where W: AspectRatio,
          C: Spatial
{
    let midpoint: FPoint3 = cube.partition().midpoint().into_space();
    let projection = {
        let mut projection = Projection::default();
        projection.far = cube.partition().width().exp() as FScalar * 2.0;
        projection
    };
    let mut camera = Camera::new(window, &projection);
    camera.look_at(&FPoint3::new(midpoint.x * 0.25, -midpoint.y, midpoint.z * 3.0),
                   &midpoint);
    camera
}

fn main() {
    let mut harness = Harness::from_glutin_window(WindowBuilder::new()
        .with_title("Bismuth")
        .with_dimensions(1024, 576)
        .with_vsync()
        .build()
        .unwrap());
    harness.start::<Bismuth>();
}
