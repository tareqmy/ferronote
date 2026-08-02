
use crate::components::{Component, DrawableComponent, EventState};
use crate::event::Event;
use crate::queue::Queue;
use ratatui::Frame;
use ratatui::layout::Rect;

/// Represents the different types of popups in the application.
#[derive(Debug, Clone)]
pub enum AnyPopup {
    Help(crate::components::help::HelpPopup),
    // About(crate::components::about::AboutPopup),
}

impl Component for AnyPopup {
    fn event(&mut self, ev: &Event) -> color_eyre::Result<EventState> {
        match self {
            AnyPopup::Help(p) => p.event(ev),
            // AnyPopup::About(p) => p.event(ev),
            // _ => Ok(EventState::NotConsumed),
        }
    }
}

impl DrawableComponent for AnyPopup {
    fn draw(&self, f: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        match self {
            AnyPopup::Help(p) => p.draw(f, area),
            // AnyPopup::About(p) => p.draw(f, area),
            // _ => Ok(()),
        }
    }
}

/// A stack of popups that handles routing events to the top-most popup.
#[derive(Debug, Clone, Default)]
pub struct PopupStack {
    stack: Vec<AnyPopup>,
    pub queue: Queue,
}

impl PopupStack {
    #[must_use]
    pub fn new(queue: Queue) -> Self {
        Self {
            stack: Vec::new(),
            queue,
        }
    }

    pub fn push(&mut self, popup: AnyPopup) {
        self.stack.push(popup);
    }

    pub fn pop(&mut self) -> Option<AnyPopup> {
        self.stack.pop()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

impl Component for PopupStack {
    fn event(&mut self, ev: &Event) -> color_eyre::Result<EventState> {
        if let Some(popup) = self.stack.last_mut() {
            popup.event(ev)
        } else {
            Ok(EventState::NotConsumed)
        }
    }
}

impl DrawableComponent for PopupStack {
    fn draw(&self, f: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        // Draw all popups in the stack, from bottom to top
        for popup in &self.stack {
            popup.draw(f, area)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::help::HelpPopup;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_popup_stack_push_pop_clear() {
        let queue = Queue::new();
        let mut stack = PopupStack::new(queue.clone());
        assert!(stack.is_empty());

        let help = HelpPopup::new(queue.clone());
        stack.push(AnyPopup::Help(help));
        assert!(!stack.is_empty());

        let popped = stack.pop();
        assert!(popped.is_some());
        assert!(stack.is_empty());

        let help2 = HelpPopup::new(queue);
        stack.push(AnyPopup::Help(help2));
        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_popup_stack_event_routing_and_draw() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let queue = Queue::new();
        let mut stack = PopupStack::new(queue.clone());

        // Event on empty stack returns NotConsumed
        let ev = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        let res = stack.event(&ev).unwrap();
        assert_eq!(res, EventState::NotConsumed);

        // Push Help popup onto stack
        let help = HelpPopup::new(queue);
        stack.push(AnyPopup::Help(help));

        // Event on non-empty stack routes to top popup
        let res_top = stack.event(&ev).unwrap();
        assert_eq!(res_top, EventState::Consumed);

        // Draw stack
        terminal.draw(|f| {
            let area = f.area();
            stack.draw(f, area).unwrap();
        }).unwrap();
    }
}
