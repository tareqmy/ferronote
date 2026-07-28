#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    SelectNote(Option<String>),
    SubmitSearch,
    SaveNote,
    DeleteNote,
    ToggleHelp,
    FileChanged(std::path::PathBuf),
    Quit,
}
