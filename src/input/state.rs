use nalgebra::{Point2, Scalar, Vector2};
use std::ops::Deref;

use event::ElementState;

pub trait State: Copy {
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
    where T: Scalar
{
    type Difference = Vector2<T>;
}

pub trait Element: Copy + Sized {
    type State: State;
}

pub trait ElementStateTransition: Copy + Sized {
    fn transition(snapshot: Self, state: Self) -> Option<Self>;
}

impl<T> ElementStateTransition for T
    where T: Copy + Eq
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

impl<E, T> InputState<E> for T
    where T: Deref,
          T::Target: InputState<E>,
          E: Element
{
    fn state(&self, element: E) -> E::State {
        self.deref().state(element)
    }
}

pub trait InputStateSnapshot {
    type Snapshot;

    fn snapshot(&mut self);
    fn as_snapshot(&self) -> &Self::Snapshot;
}

pub trait InputStateTransition<E>
    where E: Element,
          E::State: ElementStateTransition
{
    fn transition(&self, element: E) -> Option<E::State>
        where Self: InputState<E> + InputStateSnapshot,
              Self::Snapshot: InputState<E>;
}

impl<E, T> InputStateTransition<E> for T
    where T: InputState<E> + InputStateSnapshot,
          T::Snapshot: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.as_snapshot().state(element), self.state(element))
    }
}

pub trait InputStateDifference<E>: InputState<E>
    where E: Element
{
    type Difference: IntoIterator<Item = (E, <E::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference;
}
