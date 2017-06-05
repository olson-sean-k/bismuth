extern crate bismuth;
extern crate glutin;
extern crate nalgebra;

use bismuth::cube::{Cursor, Geometry, LogWidth, Root, Spatial};
use bismuth::event::{ElementState, Event, MouseButton, React};
use bismuth::framework::{self, Activity, Context, Harness, RenderContextView, RenderResult,
                         Transition, UpdateContextView, UpdateResult, WindowView};
use bismuth::input::{InputState, InputSnapshot, InputTransition, Mouse, MousePosition};
use bismuth::math::{FMatrix4, FPoint3, FScalar, IntoSpace, UPoint3, UVector3};
use bismuth::render::{Camera, MeshBuffer, MetaRenderer, Projection, ToMeshBuffer, Transform};
use glutin::WindowBuilder;
use std::marker::PhantomData;

struct State {
    pub mouse: Mouse,
}

impl State {
    pub fn new() -> Self {
        State {
            mouse: Mouse::new(),
        }
    }
}

impl framework::State for State {
}

impl React for State {
    fn react(&mut self, event: &Event) {
        self.mouse.react(event);
    }
}

struct MainActivity<R>
    where R: MetaRenderer
{
    root: Root,
    mesh: MeshBuffer,
    camera: Camera,
    phantom: PhantomData<R>,
}

impl<R> MainActivity<R>
    where R: MetaRenderer
{
    pub fn new(context: &mut Context<State, R>) -> Self {
        let root = new_root(LogWidth::new(8));
        let mesh = root.to_cube().to_mesh_buffer();
        let camera = new_camera(&context.renderer.window, &root);
        MainActivity {
            root: root,
            mesh: mesh,
            camera: camera,
            phantom: PhantomData,
        }
    }
}

impl<R> Activity<State, R> for MainActivity<R>
    where R: MetaRenderer
{
    fn update(&mut self, context: &mut UpdateContextView<State = State>) -> UpdateResult<State, R> {
        let mut dirty = false;
        if let Some(ElementState::Pressed) = context.state().mouse.transition(MouseButton::Left) {
            let ray = self.camera.cast_ray(context.window(),
                                           &context.state().mouse.state(MousePosition));
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
        context.state_mut().mouse.snapshot();
        Ok(Transition::None)
    }

    fn render(&mut self, context: &mut RenderContextView<R, State = State>) -> RenderResult {
        let mut renderer = context.renderer_mut();
        renderer.clear();
        renderer.set_transform(&Transform::new(&self.camera.transform(),
                                               &FMatrix4::identity())).unwrap();
        renderer.draw_mesh_buffer(&self.mesh);
        renderer.flush().unwrap();
        Ok(())
    }
}

impl<R> React for MainActivity<R>
    where R: MetaRenderer
{
    fn react(&mut self, event: &Event) {
        self.camera.react(event);
    }
}

fn new_root(width: LogWidth) -> Root {
    let cursor = Cursor::at_point_with_span(&UPoint3::origin(), width - 3, &UVector3::new(7, 1, 7));
    let mut root = Root::new(width);
    root.to_cube_mut().subdivide_to_cursor(&cursor);
    root
}

fn new_camera<C>(window: &WindowView, cube: &C) -> Camera
    where C: Spatial
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
    let mut harness = Harness::from_glutin_window(State::new(), WindowBuilder::new()
        .with_title("Bismuth")
        .with_dimensions(1024, 576)
        .with_vsync()
        .build()
        .unwrap());
    harness.start(|context| Box::new(MainActivity::new(context)));
}
