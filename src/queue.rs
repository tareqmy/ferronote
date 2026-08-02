use crate::action::Action;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A queue for routing actions from components up to the main application.
#[derive(Clone, Default, Debug)]
pub struct Queue {
    queue: Rc<RefCell<VecDeque<Action>>>,
}

impl Queue {
    /// Create a new, empty event queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an action to the back of the queue.
    pub fn push(&self, action: Action) {
        self.queue.borrow_mut().push_back(action);
    }

    /// Pop the next action from the front of the queue.
    #[must_use]
    pub fn pop(&self) -> Option<Action> {
        self.queue.borrow_mut().pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_fifo_order() {
        let queue = Queue::new();
        assert_eq!(queue.pop(), None);

        queue.push(Action::SaveNote);
        queue.push(Action::ToggleHelp);

        assert_eq!(queue.pop(), Some(Action::SaveNote));
        assert_eq!(queue.pop(), Some(Action::ToggleHelp));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_queue_cloned_reference_sharing() {
        let queue1 = Queue::new();
        let queue2 = queue1.clone();

        queue1.push(Action::Quit);
        assert_eq!(queue2.pop(), Some(Action::Quit));
    }
}
