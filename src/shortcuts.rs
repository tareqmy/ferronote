use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::action::Action;

/// Encapsulates a keyboard combination (key code + modifier keys).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombo {
    #[must_use]
    pub const fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
        }
    }

    #[must_use]
    pub const fn key(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[must_use]
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.code == event.code && event.modifiers.contains(self.modifiers)
    }
}

/// Central shortcut registry that governs all application keybindings,
/// key event matching, and help overlay documentation.
#[derive(Debug, Clone, Default)]
pub struct ShortcutRegistry;

impl ShortcutRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Evaluates Control modifier key combinations.
    #[must_use]
    pub fn match_control_shortcut(&self, event: &KeyEvent, is_editing: bool) -> Option<Action> {
        if !event.modifiers.contains(KeyModifiers::CONTROL) {
            return None;
        }

        match event.code {
            KeyCode::Char('k') => Some(Action::TogglePinNote),
            KeyCode::Char('v') if !is_editing => Some(Action::ToggleAbout),
            KeyCode::Char('p') => Some(Action::ToggleSettings),
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('r') => Some(Action::PromptRenameNote),
            KeyCode::Char('d') if !is_editing => Some(Action::PromptDeleteNote),
            KeyCode::Char('b') => Some(Action::ToggleNotesList),
            KeyCode::Char('e') => Some(Action::ToggleMoreShortcuts),
            KeyCode::Char('s') => Some(Action::SaveNote),
            KeyCode::Char('o') => Some(Action::OpenExternalEditor),
            KeyCode::Char('l') => Some(Action::FocusSearchBar),
            KeyCode::Char('n') => Some(Action::SaveNote),
            _ => None,
        }
    }

    /// Returns the central list of help entries `(display_key, description)` for rendering the Help overlay.
    #[must_use]
    pub fn help_entries(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("/ or Ctrl+L", "Focus search bar"),
            ("Esc", "Clear search bar / Close overlay"),
            ("Ctrl+N", "Start a new note"),
            ("Tab", "Cycle focus forwards"),
            ("Enter", "Open selected note / Create / Wiki-link"),
            ("Up / Down", "Navigate note list"),
            ("PgUp / PgDn", "Scroll note list page by page"),
            ("Home / End", "Jump to top / bottom of note list"),
            ("Ctrl+S", "Force save current note"),
            ("Ctrl+Z", "Undo in editor"),
            ("Ctrl+Y", "Redo in editor"),
            ("Ctrl+D", "Delete selected note (moves to trash)"),
            ("Ctrl+K", "Toggle Pin / Bookmark note (or 'p' in Note List)"),
            ("Ctrl+B", "Toggle Notes List panel visibility"),
            ("Ctrl+O", "Open note in external editor"),
            ("Ctrl+P", "Toggle Settings Overlay"),
            ("Ctrl+V", "Toggle About Overlay"),
            ("Mouse Drag", "Resize notes list and content panel"),
            ("?", "Toggle Help Overlay"),
            ("Ctrl+Q", "Quit Application"),
            ("", ""),
            ("h, j, k, l", "Move cursor (View mode)"),
            ("w, b", "Move word forward/backward (View mode)"),
            ("0, $", "Move to beginning/end of line (View mode)"),
            ("gg, G", "Move to beginning/end of file (View mode)"),
            ("i, a, o", "Enter Insert mode"),
            ("v, V", "Enter Visual mode (char / line)"),
            ("y, yy, yNy", "Copy (yank) selection / line / N lines"),
            ("d, dd, dNd", "Cut (delete) selection / line / N lines"),
            ("p, P", "Paste after / before cursor"),
            ("u, Ctrl+R", "Undo / Redo (Vim mode)"),
            ("/", "Search locally (regex) in file (View mode)"),
            ("n, N", "Next / Prev search match (View mode)"),
            ("Esc", "Exit back to View mode"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_combo_matching() {
        let combo = KeyCombo::ctrl('k');
        let matching_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        let non_matching_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        assert!(combo.matches(&matching_event));
        assert!(!combo.matches(&non_matching_event));
    }

    #[test]
    fn test_match_control_shortcut() {
        let registry = ShortcutRegistry::new();
        let event_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
        let event_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);

        assert_eq!(registry.match_control_shortcut(&event_k, false), Some(Action::TogglePinNote));
        assert_eq!(registry.match_control_shortcut(&event_p, false), Some(Action::ToggleSettings));
    }

    #[test]
    fn test_help_entries_non_empty() {
        let registry = ShortcutRegistry::new();
        let entries = registry.help_entries();
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|(k, _)| *k == "Ctrl+K"));
        assert!(entries.iter().any(|(k, _)| *k == "Ctrl+P"));
    }
}
