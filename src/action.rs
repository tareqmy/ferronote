/// Represents all application actions / state transitions in The Elm Architecture loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Periodic tick event for timers and background updates.
    Tick,
    /// Triggers UI redraw.
    Render,
    /// Terminal window resized to (width, height).
    Resize(u16, u16),
    /// Select a note by filename.
    SelectNote(Option<String>),
    /// Submit current search or create note prompt.
    SubmitSearch,
    /// Save active note buffer to disk.
    SaveNote,
    /// Soft delete selected note.
    DeleteNote,
    /// Toggle help modal overlay.
    ToggleHelp,
    /// Toggle settings modal overlay.
    ToggleSettings,
    /// Move focus to next setting in settings menu.
    NextSetting,
    /// Move focus to previous setting in settings menu.
    PrevSetting,
    /// Cycle active setting option (`true` for forward, `false` for backward).
    ChangeSettingOption(bool),
    /// File system change detected for file at path.
    FileChanged(std::path::PathBuf),
    /// Exit the application.
    Quit,
}

