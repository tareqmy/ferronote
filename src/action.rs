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
    ToggleSettings,
    FileChanged(std::path::PathBuf),
    Quit,
}
