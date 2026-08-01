use crate::queue::Queue;

/// Shared environment passed down to components.
#[derive(Clone, Default, Debug)]
pub struct Environment {
    pub queue: Queue,
    // Future: pub config: Rc<RefCell<Config>>,
    // Future: pub theme: Rc<RefCell<ThemePalette>>,
}

impl Environment {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }
}
