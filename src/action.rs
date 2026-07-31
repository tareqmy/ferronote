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
    /// Prompt for delete confirmation.
    PromptDeleteNote,
    /// Cancel delete confirmation.
    CancelDeleteNote,
    /// Toggle help modal overlay.
    ToggleHelp,
    /// Toggle about modal overlay.
    ToggleAbout,
    /// Toggle extra status bar shortcuts.
    ToggleMoreShortcuts,
    /// Toggle notes list visibility.
    ToggleNotesList,
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
    /// Mouse click at position (column, row).
    MouseClick(u16, u16),
    /// Mouse button pressed down at position (column, row).
    MouseDown(u16, u16),
    /// Mouse dragged to position (column, row).
    MouseDrag(u16, u16),
    /// Mouse button released at position (column, row).
    MouseUp(u16, u16),
    /// Mouse wheel scrolled up at position (column, row).
    MouseScrollUp(u16, u16),
    /// Mouse wheel scrolled down at position (column, row).
    MouseScrollDown(u16, u16),
    /// Exit the application.
    Quit,
    /// Background task found a new version.
    NewVersion(String),
    /// Trigger the update process.
    UpdateApp,
}
