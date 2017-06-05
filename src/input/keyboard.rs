use std::collections::HashSet;
use std::ops::Deref;

use event::{ElementState, Event, React, VirtualKeyCode};
use super::state::{Element, InputSnapshot, InputState, InputDifference, State};

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

impl Deref for Keyboard {
    type Target = KeyboardState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl InputDifference<VirtualKeyCode> for Keyboard {
    type Difference = Vec<(VirtualKeyCode,
                           <<VirtualKeyCode as Element>::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference {
        let mut difference = vec![];
        for key in self.state.keys.symmetric_difference(&self.snapshot.keys) {
            difference.push((*key, self.state.state(*key)));
        }
        difference
    }
}

impl InputSnapshot for Keyboard {
    type Snapshot = KeyboardState;

    fn snapshot(&mut self) {
        self.snapshot = self.state.clone();
    }

    fn as_snapshot(&self) -> &Self::Snapshot {
        &self.snapshot
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
