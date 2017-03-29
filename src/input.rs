use nalgebra::Point2;
use std::collections::HashMap;

use event::{ElementState, Event, MouseButton, Reactor};

pub struct Mouse {
    position: Point2<u32>,
    buttons: HashMap<MouseButton, ElementState>,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            position: Point2::origin(),
            buttons: HashMap::new(),
        }
    }

    pub fn position(&self) -> &Point2<u32> {
        &self.position
    }

    pub fn button(&self, button: MouseButton) -> ElementState {
        *self.buttons.get(&button).unwrap_or(&ElementState::Released)
    }
}

impl Reactor for Mouse {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::MouseInput(state, button) => {
                *self.buttons.entry(button).or_insert(state) = state;
            }
            Event::MouseMoved(x, y) => {
                self.position = Point2::new(x as u32, y as u32);
            }
            _ => {}
        }
    }
}
