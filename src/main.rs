extern crate bismuth;
extern crate glutin;
extern crate nalgebra;

use bismuth::cube::{Cursor, Geometry, LogWidth, Root, Spatial};
use bismuth::event::{ElementState, Event, MouseButton, React};
use bismuth::framework::{Application, Context, Harness, RenderContextView, UpdateContextView};
use bismuth::input::{InputStateSnapshot, InputStateTransition, Mouse};
use bismuth::math::{FMatrix4, FPoint3, FScalar, IntoSpace, UPoint3, UVector3};
use bismuth::render::{AspectRatio, Camera, MeshBuffer, MetaRenderer, Projection, ToMeshBuffer,
                      Transform};
use glutin::WindowBuilder;
use std::error;
use std::fmt;

impl<R> Application<(), R> for Bismuth
    where R: MetaRenderer
{
    type UpdateError = BismuthError;
    type RenderError = BismuthError;

    fn start(context: &mut Context<(), R>) -> Self {
        let root = new_root(LogWidth::new(8));
        let mesh = root.to_cube().to_mesh_buffer();
        let camera = new_camera(&context.renderer.window, &root);
        Bismuth {
            root: root,
            mesh: mesh,
            camera: camera,
            mouse: Mouse::new(),
        }
    }

    fn update<C>(&mut self, context: &mut C) -> Result<(), Self::UpdateError>
        where C: UpdateContextView<Data = ()>
    {
        let mut dirty = false;
        if let Some(ElementState::Pressed) = self.mouse.transition(MouseButton::Left) {
            let ray = self.camera.cast_ray(context.window(), self.mouse.position());
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
        Ok(())
    }

    fn render<C>(&mut self, context: &mut C) -> Result<(), Self::RenderError>
        where C: RenderContextView<R, Data = ()>
    {
        let mut renderer = context.renderer_mut();
        renderer.clear();
        renderer.set_transform(&Transform::new(&self.camera.transform(),
                                               &FMatrix4::identity())).unwrap();
        renderer.draw_mesh_buffer(&self.mesh);
        renderer.flush().unwrap();
        Ok(())
    }

    fn stop(self) {}
}

impl React for Bismuth {
    fn react(&mut self, event: &Event) {
        self.camera.react(event);
        self.mouse.react(event);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BismuthError;

impl fmt::Display for BismuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;

        write!(formatter, "{}", self.description())
    }
}

impl error::Error for BismuthError {
    fn description(&self) -> &str {
        ""
    }
}

struct Bismuth {
    root: Root,
    mesh: MeshBuffer,
    camera: Camera,
    mouse: Mouse,
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
    let mut harness = Harness::from_glutin_window((), WindowBuilder::new()
        .with_title("Bismuth")
        .with_dimensions(1024, 576)
        .with_vsync()
        .build()
        .unwrap());
    harness.start::<Bismuth>();
}
