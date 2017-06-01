mod keyboard;
mod mouse;
mod state;

pub use self::keyboard::{Keyboard, KeyboardState};
pub use self::mouse::{Mouse, MousePosition, MouseProximity, MouseState};
pub use self::state::{Element, InputState, InputStateDifference, InputStateSnapshot,
                      InputStateTransition, State};

pub use event::{MouseButton, VirtualKeyCode};

#[cfg(test)]
mod tests {
    use super::*;
}
