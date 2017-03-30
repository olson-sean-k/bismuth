mod mouse;
mod state;

pub use self::mouse::{Mouse, MouseState};
pub use self::state::{InputState, ToInputState, Snapshot};

#[cfg(test)]
mod tests {
    use super::*;
}
