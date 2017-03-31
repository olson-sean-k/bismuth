use event::ElementState;

pub trait Element: Copy + Sized {
    type State;
}

pub trait ToInputState<E>
    where E: Element
{
    type InputState: InputState<E>;

    fn to_state(&self) -> Self::InputState;
}

pub trait InputState<E>
    where E: Element
{
    fn state(&self, element: E) -> E::State;
}

pub trait ElementStateTransition: Copy + Sized {
    fn transition(old: Self, new: Self) -> Option<Self>;
}

impl ElementStateTransition for bool {
    fn transition(old: Self, new: Self) -> Option<Self> {
        match (old, new) {
            (false, true) => Some(true),
            (true, false) => Some(false),
            _ => None
        }
    }
}

impl ElementStateTransition for ElementState {
    fn transition(old: Self, new: Self) -> Option<Self> {
        match (old, new) {
            (ElementState::Released, ElementState::Pressed) => Some(ElementState::Pressed),
            (ElementState::Pressed, ElementState::Released) => Some(ElementState::Released),
            _ => None
        }
    }
}

pub trait InputStateSnapshot {
    type Snapshot;

    fn snapshot(&mut self);
    fn as_snapshot_state(&self) -> &Self::Snapshot;
}

pub trait InputStateTransition<E>
    where E: Element,
          E::State: ElementStateTransition
{
    fn transition(&self, element: E) -> Option<E::State>
        where Self: InputState<E> + ToInputState<E>,
              Self::InputState: InputState<E>;
}

impl<E, T> InputStateTransition<E> for T
    where T: InputState<E> + InputStateSnapshot + ToInputState<E>,
          T::InputState: InputState<E>,
          T::Snapshot: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.as_snapshot_state().state(element), self.state(element))
    }
}
