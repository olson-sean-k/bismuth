use nalgebra::Point2;
use std::collections::HashSet;

use event::{ElementState, Event, MouseButton, Reactor};
use super::state::{Element, InputState, ToInputState};

impl Element for MouseButton {
    type State = ElementState;
}

pub struct Mouse {
    position: Point2<u32>,
    buttons: MouseState,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            position: Point2::origin(),
            buttons: MouseState::new(),
        }
    }

    pub fn position(&self) -> &Point2<u32> {
        &self.position
    }
}

impl InputState<MouseButton> for Mouse {
    fn state(&self, button: MouseButton) -> ElementState {
        self.buttons.state(button)
    }
}

impl Reactor for Mouse {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::MouseInput(ElementState::Pressed, button) => {
                self.buttons.0.insert(button);
            }
            Event::MouseInput(ElementState::Released, button) => {
                self.buttons.0.remove(&button);
            }
            Event::MouseMoved(x, y) => {
                self.position = Point2::new(x as u32, y as u32);
            }
            _ => {}
        }
    }
}

impl ToInputState for Mouse {
    type Element = MouseButton;
    type InputState = MouseState;

    fn to_state(&self) -> Self::InputState {
        self.buttons.clone()
    }
}

#[derive(Clone)]
pub struct MouseState(HashSet<MouseButton>);

impl MouseState {
    pub fn new() -> Self {
        MouseState(HashSet::new())
    }
}

impl InputState<MouseButton> for MouseState {
    fn state(&self, button: MouseButton) -> ElementState {
        if self.0.contains(&button) {
            ElementState::Pressed
        }
        else {
            ElementState::Released
        }
    }
}
