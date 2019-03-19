use crate::graphics_2d::Vector;

/// An input event [Source](super::Source) that represents a mouse.
pub trait Mouse: super::Source<Event = Event> {
    /// Returns the current mouse position.
    fn position(&self) -> Vector;
}

/// A mouse button.
#[derive(Clone, Copy, Debug)]
pub enum Button {
    /// The left or primary mouse button.
    Left,
    /// The right or secondary mouse button.
    Right,
    /// The middle or tertiary mouse button. Often bound to scroll wheel depression.
    Middle,
    /// An auxiliary mouse button at the given index. Indices start at zero and represent the offset
    /// from the tertiary button i.e. ```Auxiliary(0)``` is the fourth mouse button.
    Auxiliary(u8),
}

/// A mouse state-change action
#[derive(Clone, Copy, Debug)]
pub enum Action {
    /// The return of a button from an activated state to an inactivated state.
    Up(Button),
    /// The transition of a button from an inactivated state to an activated state.
    Down(Button),
    /// A mouse movement.
    Move(Vector),
}

/// A mouse event.
#[derive(Clone, Copy, Debug)]
pub struct Event {
    /// The associated action.
    pub action: Action,
    /// The mouse position at the time of the event.
    pub position: Vector,
}

impl super::Event for Event {}