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
    /// Prompt for renaming the selected note.
    PromptRenameNote,
    /// Submit the new name for the note.
    SubmitRenameNote(String),
    /// Cancel renaming.
    CancelRenameNote,
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
    /// Move focus to first setting in settings menu (Home).
    FirstSetting,
    /// Move focus to last setting in settings menu (End).
    LastSetting,
    /// Move focus page up in settings menu (PageUp).
    PageUpSetting,
    /// Move focus page down in settings menu (PageDown).
    PageDownSetting,
    /// Cycle the currently selected setting (true for forward, false for backward).
    ChangeSettingOption(bool),
    /// Type a character into a text setting field.
    SettingsTypeChar(char),
    /// Backspace a character from a text setting field.
    SettingsBackspace,
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
    /// Toggle editor view mode / edit mode.
    ToggleEditMode,
    /// Exit the application.
    Quit,
    /// Background task found a new version.
    NewVersion(String),
    /// Open the current note in an external editor.
    OpenExternalEditor,
    /// Trigger the update process.
    UpdateApp,
    /// Toggle pinned status of selected note.
    TogglePinNote,
    /// Move focus to search bar.
    FocusSearchBar,
}
