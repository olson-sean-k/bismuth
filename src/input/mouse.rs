use nalgebra::Point2;
use std::collections::HashSet;

use event::{ElementState, Event, MouseButton, Reactor};
use super::state::{Element, InputState, InputStateSnapshot, ToInputState};

impl Element for MouseButton {
    type State = ElementState;
}

#[derive(Clone, Copy)]
pub struct MouseProximity;

impl Element for MouseProximity {
    type State = bool;
}

pub struct Mouse {
    position: Point2<u32>,
    state: MouseState,
    snapshot: MouseState,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            position: Point2::origin(),
            state: MouseState::new(),
            snapshot: MouseState::new(),
        }
    }

    pub fn position(&self) -> &Point2<u32> {
        &self.position
    }
}

impl InputState<MouseButton> for Mouse {
    fn state(&self, button: MouseButton) -> ElementState {
        self.state.state(button)
    }
}

impl InputState<MouseProximity> for Mouse {
    fn state(&self, proximity: MouseProximity) -> bool {
        self.state.state(proximity)
    }
}

impl Reactor for Mouse {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::MouseEntered => {
                self.state.proximity = true;
            }
            Event::MouseInput(ElementState::Pressed, button) => {
                self.state.buttons.insert(button);
            }
            Event::MouseInput(ElementState::Released, button) => {
                self.state.buttons.remove(&button);
            }
            Event::MouseLeft => {
                self.state.proximity = false;
            }
            Event::MouseMoved(x, y) => {
                self.position = Point2::new(x as u32, y as u32);
            }
            _ => {}
        }
    }
}

impl InputStateSnapshot for Mouse {
    type Snapshot = MouseState;

    fn snapshot(&mut self) {
        self.snapshot = self.to_state();
    }

    fn as_snapshot_state(&self) -> &Self::Snapshot {
        &self.snapshot
    }
}

impl ToInputState<MouseButton> for Mouse {
    type InputState = MouseState;

    fn to_state(&self) -> Self::InputState {
        self.state.clone()
    }
}

#[derive(Clone)]
pub struct MouseState {
    buttons: HashSet<MouseButton>,
    proximity: bool,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            buttons: HashSet::new(),
            proximity: false,
        }
    }
}

impl InputState<MouseButton> for MouseState {
    fn state(&self, button: MouseButton) -> ElementState {
        if self.buttons.contains(&button) {
            ElementState::Pressed
        }
        else {
            ElementState::Released
        }
    }
}

impl InputState<MouseProximity> for MouseState {
    fn state(&self, _: MouseProximity) -> bool {
        self.proximity
    }
}
