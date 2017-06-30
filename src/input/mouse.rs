use boolinator::Boolinator;
use nalgebra::Point2;
use num::Zero;
use std::collections::HashSet;
use std::ops::Deref;

use event::{ElementState, Event, MouseButton, React};
use super::state::{CompositeState, Element, Input, InputState, InputDifference, InputTransition,
                   State};

impl Element for MouseButton {
    type State = ElementState;
}

#[derive(Clone, Copy)]
pub struct MousePosition;

impl Element for MousePosition {
    type State = Point2<i32>;
}

#[derive(Clone, Copy)]
pub struct MouseProximity;

impl Element for MouseProximity {
    type State = bool;
}

pub struct Mouse {
    now: MouseState,
    previous: MouseState,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse::default()
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            now: MouseState::new(),
            previous: MouseState::new(),
        }
    }
}

impl Deref for Mouse {
    type Target = MouseState;

    fn deref(&self) -> &Self::Target {
        &self.now
    }
}

impl Input for Mouse {
    type State = MouseState;

    fn now(&self) -> &Self::State {
        &self.now
    }

    fn previous(&self) -> &Self::State {
        &self.previous
    }

    fn snapshot(&mut self) {
        self.previous = self.now.clone();
    }
}

impl InputDifference<MousePosition> for Mouse {
    type Difference = Option<
        (
            MousePosition,
            <<MousePosition as Element>::State as State>::Difference,
        ),
    >;

    // This is distinct from `InputTransition::transition`. That function
    // indicates whether or not a change has occurred and yields the current
    // state. This function instead yields a *difference*, for which the type
    // representing the change in state can be entirely different than the type
    // of the state itself. For mouse position, `transition` yields a point and
    // `difference` yields a vector.
    fn difference(&self) -> Self::Difference {
        let difference = self.now.state(MousePosition) - self.previous.state(MousePosition);
        (!difference.is_zero()).as_some((MousePosition, difference))
    }
}

impl InputDifference<MouseProximity> for Mouse {
    type Difference = Option<
        (
            MouseProximity,
            <<MouseProximity as Element>::State as State>::Difference,
        ),
    >;

    fn difference(&self) -> Self::Difference {
        self.transition(MouseProximity)
            .map(|state| (MouseProximity, state))
    }
}

impl React for Mouse {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::MouseEntered => {
                self.now.proximity = true;
            }
            Event::MouseInput(ElementState::Pressed, button) => {
                self.now.buttons.insert(button);
            }
            Event::MouseInput(ElementState::Released, button) => {
                self.now.buttons.remove(&button);
            }
            Event::MouseLeft => {
                self.now.proximity = false;
            }
            Event::MouseMoved(x, y) => {
                self.now.position = Point2::new(x, y);
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct MouseState {
    buttons: HashSet<MouseButton>,
    position: Point2<i32>,
    proximity: bool,
}

impl MouseState {
    fn new() -> Self {
        MouseState {
            buttons: HashSet::new(),
            position: Point2::origin(),
            proximity: false,
        }
    }
}

impl CompositeState<MouseButton> for MouseState {
    type Composite = HashSet<MouseButton>;

    fn composite(&self) -> &Self::Composite {
        &self.buttons
    }
}

impl InputState<MousePosition> for MouseState {
    fn state(&self, _: MousePosition) -> <MousePosition as Element>::State {
        self.position
    }
}

impl InputState<MouseProximity> for MouseState {
    fn state(&self, _: MouseProximity) -> <MouseProximity as Element>::State {
        self.proximity
    }
}
