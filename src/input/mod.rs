mod keyboard;
mod mouse;
mod state;

pub use self::keyboard::{Keyboard, KeyboardState};
pub use self::mouse::{Mouse, MouseState};
pub use self::state::{Element, InputState, InputStateDifference, InputStateSnapshot,
                      InputStateTransition};

#[cfg(test)]
mod tests {
    use super::*;
}
