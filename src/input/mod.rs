mod keyboard;
mod mouse;
mod state;

pub use self::keyboard::{Keyboard, KeyboardState};
pub use self::mouse::{Mouse, MousePosition, MouseProximity, MouseState};
pub use self::state::{InputState, InputDifference, InputTransition, Snapshot};

pub use event::{MouseButton, VirtualKeyCode};

#[cfg(test)]
mod tests {
    use super::*;
}
