use std::collections::HashSet;
use std::ops::Deref;

use event::{ElementState, Event, React, VirtualKeyCode};
use input::state::{CompositeState, Element, Input, Snapshot};

impl Element for VirtualKeyCode {
    type State = ElementState;
}

/// Keyboard input device.
pub struct Keyboard {
    live: KeyboardState,
    snapshot: KeyboardState,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard::default()
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard {
            live: KeyboardState::new(),
            snapshot: KeyboardState::new(),
        }
    }
}

impl Deref for Keyboard {
    type Target = KeyboardState;

    fn deref(&self) -> &Self::Target {
        &self.live
    }
}

impl Input for Keyboard {
    type State = KeyboardState;

    fn live(&self) -> &Self::State {
        &self.live
    }

    fn snapshot(&self) -> &Self::State {
        &self.snapshot
    }
}

impl React for Keyboard {
    fn react(&mut self, event: &Event) {
        if let Event::KeyboardInput(state, _, key) = *event {
            if let Some(key) = key {
                match state {
                    ElementState::Pressed => {
                        self.live.keys.insert(key);
                    }
                    ElementState::Released => {
                        self.live.keys.remove(&key);
                    }
                }
            }
        }
    }
}

impl Snapshot for Keyboard {
    fn snapshot(&mut self) {
        self.snapshot = self.live.clone();
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

impl CompositeState<VirtualKeyCode> for KeyboardState {
    type Composite = HashSet<VirtualKeyCode>;

    fn composite(&self) -> &Self::Composite {
        &self.keys
    }
}
