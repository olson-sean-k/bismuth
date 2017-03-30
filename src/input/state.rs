use std::ops::{Deref, DerefMut};

use event::{ElementState, Event, Reactor};

pub trait ToInputState {
    type Element: Copy;
    type State: InputState<Self::Element>;

    fn to_state(&self) -> Self::State;
}

pub trait InputState<E>
    where E: Copy
{
    fn state(&self, element: E) -> ElementState;
}

pub trait ElementStateTransition: Sized {
    fn transition(old: &Self, new: &Self) -> Option<Self>;
}

impl ElementStateTransition for ElementState {
    fn transition(old: &Self, new: &Self) -> Option<Self> {
        match (*old, *new) {
            (ElementState::Released, ElementState::Pressed) => Some(ElementState::Pressed),
            (ElementState::Pressed, ElementState::Released) => Some(ElementState::Released),
            _ => None
        }
    }
}

pub struct Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    input: T,
    state: T::State,
}

impl<E, T> Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    pub fn new(input: T) -> Self {
        Snapshot {
            state: input.to_state(),
            input: input,
        }
    }

    pub fn transition(&self, element: T::Element) -> Option<ElementState> {
        match (self.state.state(element), self.input.state(element)) {
            (ElementState::Released, ElementState::Pressed) => Some(ElementState::Pressed),
            (ElementState::Pressed, ElementState::Released) => Some(ElementState::Released),
            _ => None
        }
    }

    pub fn snapshot(&mut self) {
        self.state = self.input.to_state();
    }
}

impl<E, T> Deref for Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<E, T> DerefMut for Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl<E, T> Reactor for Snapshot<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    fn react(&mut self, event: &Event) {
        self.input.react(event);
    }
}