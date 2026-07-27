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
