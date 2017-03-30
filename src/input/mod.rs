mod mouse;
mod state;

pub use self::mouse::{Mouse, MouseState};
pub use self::state::{ElementTransition, InputState, ToInputState};

#[cfg(test)]
mod tests {
    use super::*;
}
