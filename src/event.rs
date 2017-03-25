use glutin::Window;

pub use winit::{ElementState, Event, MouseButton, MouseCursor, MouseScrollDelta, TouchPhase,
                VirtualKeyCode};

pub trait PollEvents {
    type Output: IntoIterator<Item = Event>;

    fn poll_events(&self) -> Self::Output;
}

impl PollEvents for Window {
    type Output = Vec<Event>;

    fn poll_events(&self) -> Self::Output {
        self.poll_events().collect()
    }
}
