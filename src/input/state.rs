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

pub struct ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    input: T,
    state: T::State,
}

impl<E, T> ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    pub fn new(input: T) -> Self {
        ElementTransition {
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

impl<E, T> Deref for ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<E, T> DerefMut for ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl<E, T> Reactor for ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: InputState<E>,
          E: Copy
{
    fn react(&mut self, event: &Event) {
        self.input.react(event);
    }
}
