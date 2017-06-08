use nalgebra::{Point2, Scalar, Vector2};
use std::collections::HashSet;
use std::hash::Hash;

use event::{ElementState, React};

pub trait State: Copy + Eq {
    // TODO: Use a default type (`Self`) here once that feature stabilizes.
    type Difference/* = Self*/;

    fn transition(state: Self, snapshot: Self) -> Option<Self> {
        if state == snapshot {
            None
        }
        else {
            Some(state)
        }
    }
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

pub trait CompositeState<E>
    where E: Element
{
    // TODO: Use a default type (`E::State`) here once that feature stabilizes.
    type Composite/* = E::State*/;

    fn composite(&self) -> &Self::Composite;
}

pub trait InputState<E>
    where E: Element
{
    fn state(&self, element: E) -> E::State;
}

impl<E, T> InputState<E> for T
    where T: CompositeState<E, Composite = HashSet<E>>,
          E: Element<State = ElementState> + Eq + Hash
{
    fn state(&self, element: E) -> E::State {
        if self.composite().contains(&element) {
            ElementState::Pressed
        }
        else {
            ElementState::Released
        }
    }
}

pub trait InputTransition<E>
    where E: Element
{
    fn transition(&self, element: E) -> Option<E::State>;
}

impl<E, T> InputTransition<E> for T
    where T: Input,
          T::State: InputState<E>,
          E: Element
{
    fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.now().state(element), self.previous().state(element))
    }
}

pub trait InputDifference<E>
    where E: Element
{
    type Difference: IntoIterator<Item = (E, <E::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference;
}

impl<E, S, T> InputDifference<E> for T
    where T: Input,
          T::State: CompositeState<E, Composite = HashSet<E>> + InputState<E>,
          E: Element<State = S> + Eq + Hash,
          S: State<Difference = S>
{
    type Difference = Vec<(E, <E::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference {
        self.now().composite().symmetric_difference(self.previous().composite())
            .map(|element| (*element, self.now().state(*element)))
            .collect()
    }
}

pub trait Input: React {
    type State;

    fn now(&self) -> &Self::State;
    fn previous(&self) -> &Self::State;

    fn snapshot(&mut self);
}
