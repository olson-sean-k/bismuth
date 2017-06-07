use std::collections::HashSet;

use event::{ElementState, Event, React, VirtualKeyCode};
use super::state::{Element, Input, InputComposite, InputState};

impl Element for VirtualKeyCode {
    type State = ElementState;
}

pub struct Keyboard {
    state: KeyboardState,
    snapshot: KeyboardState,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            state: KeyboardState::new(),
            snapshot: KeyboardState::new(),
        }
    }
}

impl Input for Keyboard {
    type State = KeyboardState;

    fn now(&self) -> &Self::State {
        &self.state
    }

    fn previous(&self) -> &Self::State {
        &self.snapshot
    }

    fn snapshot(&mut self) {
        self.snapshot = self.state.clone();
    }
}

impl React for Keyboard {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::KeyboardInput(state, _, key) => {
                if let Some(key) = key {
                    match state {
                        ElementState::Pressed => {
                            self.state.keys.insert(key);
                        }
                        ElementState::Released => {
                            self.state.keys.remove(&key);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct KeyboardState {
    keys: HashSet<VirtualKeyCode>,
}

impl KeyboardState {
    fn new() -> Self {
        KeyboardState {
            keys: HashSet::new(),
        }
    }
}

impl InputComposite<VirtualKeyCode> for KeyboardState {
    type Composite = HashSet<VirtualKeyCode>;

    fn composite(&self) -> &Self::Composite {
        &self.keys
    }
}

impl InputState<VirtualKeyCode> for KeyboardState {
    fn state(&self, key: VirtualKeyCode) -> <VirtualKeyCode as Element>::State {
        if self.keys.contains(&key) {
            ElementState::Pressed
        }
        else {
            ElementState::Released
        }
    }
}
