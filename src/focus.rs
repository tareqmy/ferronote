#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    SearchBar,
    NoteList,
    Editor,
}

impl Focus {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::SearchBar => Self::NoteList,
            Self::NoteList => Self::Editor,
            Self::Editor => Self::SearchBar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_next_cycling() {
        let mut focus = Focus::default();
        assert_eq!(focus, Focus::SearchBar);

        focus = focus.next();
        assert_eq!(focus, Focus::NoteList);

        focus = focus.next();
        assert_eq!(focus, Focus::Editor);

        focus = focus.next();
        assert_eq!(focus, Focus::SearchBar);
    }
}

