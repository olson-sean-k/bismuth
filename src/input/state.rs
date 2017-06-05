use nalgebra::{Point2, Scalar, Vector2};
use std::ops::Deref;

use event::ElementState;

pub trait State: Copy + Eq {
    // TODO: Use a default type (`Self`) here once that feature stabilizes.
    type Difference/* = Self*/;
}

impl State for bool {
    type Difference = Self;
}

impl State for ElementState {
    type Difference = Self;
}

impl<T> State for Point2<T>
    where T: Eq + Scalar
{
    type Difference = Vector2<T>;
}

pub trait Element: Copy + Sized {
    type State: State;
}

pub trait StateTransition: Copy + Sized {
    fn transition(snapshot: Self, state: Self) -> Option<Self>;
}

impl<T> StateTransition for T
    where T: State
{
    fn transition(snapshot: Self, state: Self) -> Option<Self> {
        if snapshot == state {
            None
        }
        else {
            Some(state)
        }
    }
}

pub trait InputState<E>
    where E: Element
{
    fn state(&self, element: E) -> E::State;
}

// This is dubious. It allows input device types to implement `InputState` in
// terms of a state member that already implements `InputState` (by yielding it
// in a `Deref` implementation). The alternative is to re-implement
// `InputState` for each input device type.
impl<E, T> InputState<E> for T
    where T: Deref,
          T::Target: InputState<E>,
          E: Element
{
    fn state(&self, element: E) -> E::State {
        self.deref().state(element)
    }
}

pub trait InputSnapshot {
    type Snapshot;

    fn snapshot(&mut self);
    fn as_snapshot(&self) -> &Self::Snapshot;
}

pub trait InputTransition<E>
    where E: Element,
          E::State: StateTransition
{
    fn transition(&self, element: E) -> Option<E::State>
        where Self: InputState<E> + InputSnapshot,
              Self::Snapshot: InputState<E>;
}

impl<E, T> InputTransition<E> for T
    where T: InputState<E> + InputSnapshot,
          T::Snapshot: InputState<E>,
          E: Element,
          E::State: StateTransition
{
    fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.as_snapshot().state(element), self.state(element))
    }
}

pub trait InputDifference<E>: InputState<E>
    where E: Element
{
    type Difference: IntoIterator<Item = (E, <E::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference;
}
