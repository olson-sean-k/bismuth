use bit_vec::BitVec;

use event::{ElementState, Event, Reactor, VirtualKeyCode};
use super::state::{Element, InputState, InputStateSnapshot};

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

impl InputState<VirtualKeyCode> for Keyboard {
    fn state(&self, key: VirtualKeyCode) -> ElementState {
        self.state.state(key)
    }
}

impl InputStateSnapshot for Keyboard {
    type Snapshot = KeyboardState;

    fn snapshot(&mut self) {
        self.snapshot = self.state.clone();
    }

    fn as_snapshot(&self) -> &Self::Snapshot {
        &self.snapshot
    }
}

impl Reactor for Keyboard {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::KeyboardInput(state, _, key) => {
                if let Some(key) = key {
                    let state = if let ElementState::Pressed = state { true } else { false };
                    self.state.keys.set(key as usize, state);
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct KeyboardState {
    keys: BitVec,
}

impl KeyboardState {
    fn new() -> Self {
        KeyboardState {
            keys: BitVec::from_elem(150, false), // There are 150 virtual key codes.
        }
    }
}

impl InputState<VirtualKeyCode> for KeyboardState {
    fn state(&self, key: VirtualKeyCode) -> ElementState {
        // `key` should never be out of bounds, so just `unwrap` the `Option`.
        if self.keys.get(key as usize).unwrap() {
            ElementState::Pressed
        }
        else {
            ElementState::Released
        }
    }
}
