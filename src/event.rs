use glutin::Window;

pub use winit::{ElementState, Event, MouseButton, MouseCursor, MouseScrollDelta, ScanCode,
                TouchPhase, VirtualKeyCode};

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

pub trait React {
    fn react(&mut self, event: &Event);
}

impl React for () {
    fn react(&mut self, _: &Event) {}
}
