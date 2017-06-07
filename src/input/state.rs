use nalgebra::{Point2, Scalar, Vector2};
use std::collections::HashSet;
use std::hash::Hash;

use event::{ElementState, React};

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

pub trait InputComposite<E>
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

pub trait InputTransition<E>
    where E: Element,
          E::State: StateTransition
{
    fn transition(&self, element: E) -> Option<E::State>;
}

impl<E, T> InputTransition<E> for T
    where T: Input,
          T::State: InputState<E>,
          E: Element,
          E::State: StateTransition
{
    fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.previous().state(element), self.now().state(element))
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
          T::State: InputComposite<E, Composite = HashSet<E>> + InputState<E>,
          E: Element<State = S> + Eq + Hash,
          S: State<Difference = S>
{
    type Difference = Vec<(E, <E::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference {
        self.now().composite().symmetric_difference(self.previous().composite())
            .map(|element| (*element, self.state(*element))).collect()
    }
}

pub trait Input: React {
    type State;

    fn now(&self) -> &Self::State;
    fn previous(&self) -> &Self::State;

    fn snapshot(&mut self);
}

impl<E, T> InputState<E> for T
    where T: Input,
          T::State: InputState<E>,
          E: Element
{
    fn state(&self, element: E) -> E::State {
        self.now().state(element)
    }
}
