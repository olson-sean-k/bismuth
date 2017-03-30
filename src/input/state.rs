use std::ops::{Deref, DerefMut};

use event::{ElementState, Event, Reactor};

pub trait Element: Copy + Sized {
    type State;
}

pub trait ToInputState {
    type Element: Element;
    type InputState: InputState<Self::Element>;

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

pub struct Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::InputState: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    input: T,
    state: T::InputState,
}

impl<E, T> Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::InputState: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    pub fn new(input: T) -> Self {
        Snapshot {
            state: input.to_state(),
            input: input,
        }
    }

    pub fn transition(&self, element: E) -> Option<E::State> {
        E::State::transition(self.state.state(element), self.input.state(element))
    }

    pub fn snapshot(&mut self) {
        self.state = self.input.to_state();
    }
}

impl<E, T> Deref for Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::InputState: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<E, T> DerefMut for Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::InputState: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl<E, T> Reactor for Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::InputState: InputState<E>,
          E: Element,
          E::State: ElementStateTransition
{
    fn react(&mut self, event: &Event) {
        self.input.react(event);
    }
}
