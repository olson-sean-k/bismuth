//! This module provides an interface for querying user input devices like
//! keyboards and mice.
//!
//! Input devices track two stages of state: live state and snapshot state.
//! Live state and snapshot state are updated via the `React` and `Snapshot`
//! traits, respectively. This module provides access to the live state to
//! query the current state of a device and also provides traits that compare
//! live state to snapshot state.

mod keyboard;
mod mouse;
mod state;

pub use self::keyboard::{Keyboard, KeyboardState};
pub use self::mouse::{Mouse, MousePosition, MouseProximity, MouseState};
pub use self::state::{InputState, InputDifference, InputTransition, Snapshot};

pub use event::{MouseButton, VirtualKeyCode};
