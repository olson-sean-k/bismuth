use nalgebra::Point2;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use event::{ElementState, Event, MouseButton, Reactor};

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
          T::State: Default + InputState<E>,
          E: Copy
{
    pub fn new(input: T) -> Self {
        ElementTransition {
            input: input,
            state: T::State::default(),
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
          T::State: Default + InputState<E>,
          E: Copy
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl<E, T> DerefMut for ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: Default + InputState<E>,
          E: Copy
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl<E, T> Reactor for ElementTransition<E, T>
    where T: InputState<E> + Reactor + ToInputState<Element = E>,
          T::State: Default + InputState<E>,
          E: Copy
{
    fn react(&mut self, event: &Event) {
        self.input.react(event);
    }
}

pub struct Mouse {
    position: Point2<u32>,
    buttons: MouseState,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            position: Point2::origin(),
            buttons: MouseState::new(),
        }
    }

    pub fn position(&self) -> &Point2<u32> {
        &self.position
    }
}

impl InputState<MouseButton> for Mouse {
    fn state(&self, button: MouseButton) -> ElementState {
        self.buttons.state(button)
    }
}

impl Reactor for Mouse {
    fn react(&mut self, event: &Event) {
        match *event {
            Event::MouseInput(ElementState::Pressed, button) => {
                self.buttons.0.insert(button);
            }
            Event::MouseInput(ElementState::Released, button) => {
                self.buttons.0.remove(&button);
            }
            Event::MouseMoved(x, y) => {
                self.position = Point2::new(x as u32, y as u32);
            }
            _ => {}
        }
    }
}

impl ToInputState for Mouse {
    type Element = MouseButton;
    type State = MouseState;

    fn to_state(&self) -> Self::State {
        self.buttons.clone()
    }
}

#[derive(Clone)]
pub struct MouseState(HashSet<MouseButton>);

impl MouseState {
    pub fn new() -> Self {
        MouseState(HashSet::new())
    }
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState::new()
    }
}

impl InputState<MouseButton> for MouseState {
    fn state(&self, button: MouseButton) -> ElementState {
        if self.0.contains(&button) {
            ElementState::Pressed
        }
        else {
            ElementState::Released
        }
    }
}
