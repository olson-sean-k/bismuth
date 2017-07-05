use nalgebra::{Point2, Scalar, Vector2};
use std::collections::HashSet;
use std::hash::Hash;

use event::{ElementState, React};

/// An atomic state of an input element.
pub trait State: Copy + Eq {
    // TODO: Use a default type (`Self`) here once that feature stabilizes.
    /// Representation of a difference between states.
    type Difference;

    /// Gets the transition between a live and snapshot state. If no transition
    /// has occurred, returns `None`.
    fn transition(live: Self, snapshot: Self) -> Option<Self> {
        if live == snapshot {
            None
        }
        else {
            Some(live)
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
where
    T: Eq + Scalar,
{
    type Difference = Vector2<T>;
}

/// An input element, such as a button, key, or position.
pub trait Element: Copy + Sized {
    /// Representation of the state of the element.
    type State: State;
}

/// A state with a composite representation. This is used for input elements
/// which have a cardinality greater than one. For example, a mouse may have
/// more than one button.
pub trait CompositeState<E>
where
    E: Element,
{
    // TODO: Use a default type (`E::State`) here once that feature stabilizes.
    /// Representation of the composite state.
    type Composite;

    /// Gets the composite state.
    fn composite(&self) -> &Self::Composite;
}

/// Provides a state for an input element.
pub trait InputState<E>
where
    E: Element,
{
    /// Gets the state of an input element.
    fn state(&self, element: E) -> E::State;
}

// Blanket implementation for `InputState` for composite states represented by
// a `HashSet`, such as keys and buttons.
impl<E, T> InputState<E> for T
where
    T: CompositeState<E, Composite = HashSet<E>>,
    E: Element<State = ElementState> + Eq + Hash,
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

/// Provides a transition state for an input element.
pub trait InputTransition<E>
where
    E: Element,
{
    /// Gets the transition state of an input element.
    fn transition(&self, element: E) -> Option<E::State>;
}

impl<E, T> InputTransition<E> for T
where
    T: Input,
    T::State: InputState<E>,
    E: Element,
{
    fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.live().state(element), self.snapshot().state(element))
    }
}

/// Determines the difference in state for an input element.
pub trait InputDifference<E>
where
    E: Element,
{
    /// Iterable representation of differences in state.
    type Difference: IntoIterator<Item = (E, <E::State as State>::Difference)>;

    /// Gets the difference in state for an input element.
    fn difference(&self) -> Self::Difference;
}

// Blanket implementation for `InputDifference` for composite states
// represented by a `HashSet`, such as keys and buttons.
impl<E, S, T> InputDifference<E> for T
where
    T: Input,
    T::State: CompositeState<E, Composite = HashSet<E>> + InputState<E>,
    E: Element<State = S> + Eq + Hash,
    S: State<Difference = S>,
{
    type Difference = Vec<(E, <E::State as State>::Difference)>;

    fn difference(&self) -> Self::Difference {
        self.live()
            .composite()
            .symmetric_difference(self.snapshot().composite())
            .map(|element| (*element, self.live().state(*element)))
            .collect()
    }
}

/// An input device with a live state and snapshot state. These are updated via
/// `React` and `Snapshot` and provide information about the live state and
/// changes based on the snapshot state.
pub trait Input: React + Snapshot {
    /// Aggregate state for the input device.
    type State;

    /// Gets the live state.
    fn live(&self) -> &Self::State;
    // TODO: The term "snapshot" is ambiguous. Here, it refers to the snapshot
    //       of the state of an input device. In the `Snapshot` trait, it is
    //       used as a verb for the operation of taking a snapshot (copying the
    //       live state into the snapshot state). However, the `Input` trait is
    //       not exposed outside of this module, so this shouldn't affect
    //       client code.
    /// Gets the snapshot state.
    fn snapshot(&self) -> &Self::State;
}

/// Provides snapshotting for an input device. Input devices maintain a live
/// state and snapshot state, which are updated via `React` and this trait,
/// respectively.
pub trait Snapshot {
    /// Snapshots the live state.
    fn snapshot(&mut self);
}
