mod mouse;
mod state;

pub use self::mouse::{Mouse, MouseState};
pub use self::state::{Element, InputState, InputStateSnapshot, InputStateTransition, ToInputState};

#[cfg(test)]
mod tests {
    use super::*;
}
