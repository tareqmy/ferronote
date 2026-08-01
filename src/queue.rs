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
