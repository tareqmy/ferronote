pub mod editor;
pub mod note_list;
pub mod search_bar;
pub mod help;
pub mod popup_stack;

use crate::event::Event;
use color_eyre::Result;
use ratatui::{layout::Rect, Frame};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CommandBlocking {
    Blocking,
    PassingOn,
}

/// Base component trait for UI elements
pub trait Component {
    /// Handle an event, returning whether it was consumed
    fn event(&mut self, _ev: &Event) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }

    /// Focus or unfocus the component
    fn focus(&mut self, _focus: bool) {}

    /// Is the component currently visible?
    fn is_visible(&self) -> bool {
        true
    }

    /// Hide the component
    fn hide(&mut self) {}

    /// Show the component
    fn show(&mut self) -> Result<()> {
        Ok(())
    }

    /// Toggle visibility
    fn toggle_visible(&mut self) -> Result<()> {
        if self.is_visible() {
            self.hide();
            Ok(())
        } else {
            self.show()
        }
    }
}

/// Trait for components that can be drawn to the screen
pub trait DrawableComponent {
    /// Draw the component onto the frame within the specified rect
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()>;
}
