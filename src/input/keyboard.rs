use std::collections::HashSet;
use std::ops::Deref;

use event::{ElementState, Event, React, VirtualKeyCode};
use super::state::{CompositeState, Element, Input};

impl Element for VirtualKeyCode {
    type State = ElementState;
}

pub struct Keyboard {
    now: KeyboardState,
    previous: KeyboardState,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard::default()
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            now: KeyboardState::new(),
            previous: KeyboardState::new(),
        }
    }
}

impl Deref for Keyboard {
    type Target = KeyboardState;

    fn deref(&self) -> &Self::Target {
        &self.now
    }
}

impl Input for Keyboard {
    type State = KeyboardState;

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

impl React for Keyboard {
    fn react(&mut self, event: &Event) {
        if let Event::KeyboardInput(state, _, key) = *event {
            if let Some(key) = key {
                match state {
                    ElementState::Pressed => {
                        self.now.keys.insert(key);
                    }
                    ElementState::Released => {
                        self.now.keys.remove(&key);
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct KeyboardState {
    keys: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    fn new() -> Self {
        KeyboardState { keys: HashSet::new() }
    }
}

impl CompositeState<VirtualKeyCode> for KeyboardState {
    type Composite = HashSet<VirtualKeyCode>;

    fn composite(&self) -> &Self::Composite {
        &self.keys
    }
}
