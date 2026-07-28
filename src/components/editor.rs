use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
};
use std::collections::HashMap;
use std::time::Instant;
use tui_textarea::TextArea;

/// Main note editor component supporting Markdown editing, undo/redo, and wiki-link extraction.
#[derive(Debug, Clone)]
pub struct Editor<'a> {
    /// In-memory text areas per open note filename.
    pub textareas: HashMap<String, TextArea<'a>>,
    /// Active note filename being edited.
    pub current_note: Option<String>,
    /// Last edit timestamp used for debounced auto-save calculation.
    pub last_edit_time: Option<Instant>,
    /// Last saved content per note filename for dirty checking.
    pub original_content: HashMap<String, String>,
}

impl Default for Editor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor<'_> {
    /// Creates a new empty `Editor` component.
    #[must_use]
    pub fn new() -> Self {
        Self {
            textareas: HashMap::new(),
            current_note: None,
            last_edit_time: None,
            original_content: HashMap::new(),
        }
    }


    pub fn set_content(&mut self, title: &str, content: &str) {
        if title.is_empty() {
            self.current_note = None;
            return;
        }

        self.current_note = Some(title.to_string());

        if !self.textareas.contains_key(title) {
            let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
            let ta = TextArea::new(lines);
            self.textareas.insert(title.to_string(), ta);
            self.original_content
                .insert(title.to_string(), content.to_string());
        } else {
            // Check if we need to reload due to external change (e.g. content doesn't match and no unsaved changes)
            if !self.has_unsaved_changes() && self.content() != content {
                let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
                let ta = TextArea::new(lines);
                self.textareas.insert(title.to_string(), ta);
                self.original_content
                    .insert(title.to_string(), content.to_string());
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(ref note) = self.current_note {
            if let Some(ta) = self.textareas.get_mut(note) {
                let modified = if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('z')
                {
                    ta.undo()
                } else if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('y')
                {
                    ta.redo()
                } else {
                    ta.input(key)
                };

                if modified {
                    self.last_edit_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        area: Rect,
        is_focused: bool,
        theme: &crate::theme::ThemePalette,
    ) {
        let title = if let Some(ref note) = self.current_note {
            format!(" {} ", note.strip_suffix(".md").unwrap_or(note))
        } else {
            " Editor ".to_string()
        };

        let border_color = if is_focused {
            theme.border_active
        } else {
            theme.border_inactive
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(theme.title)));

        if let Some(ref note) = self.current_note {
            if let Some(ta) = self.textareas.get(note) {
                let mut ta_clone = ta.clone();
                ta_clone.set_block(block);
                if is_focused {
                    ta_clone.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
                } else {
                    ta_clone.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
                }
                frame.render_widget(&ta_clone, area);
                return;
            }
        }

        // Empty state
        let mut empty_ta = TextArea::default();
        empty_ta.set_block(block);
        empty_ta.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
        frame.render_widget(&empty_ta, area);
    }

    #[must_use]
    pub fn content(&self) -> String {
        if let Some(ref note) = self.current_note {
            if let Some(ta) = self.textareas.get(note) {
                return ta.lines().join("\n");
            }
        }
        String::new()
    }

    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        if let Some(ref note) = self.current_note {
            if let Some(orig) = self.original_content.get(note) {
                return orig != &self.content();
            }
        }
        false
    }

    pub fn mark_saved(&mut self) {
        if let Some(ref note) = self.current_note {
            self.original_content
                .insert(note.to_string(), self.content());
            self.last_edit_time = None;
        }
    }

    #[must_use]
    pub fn word_count(&self) -> usize {
        self.content().split_whitespace().count()
    }

    #[must_use]
    pub fn char_count(&self) -> usize {
        // Exclude newlines from char count to be more accurate to typical editors
        self.content()
            .chars()
            .filter(|c| !c.is_whitespace() || *c == ' ')
            .count()
    }

    pub fn extract_wiki_link_at_cursor(&self) -> Option<String> {
        if let Some(ref note) = self.current_note {
            if let Some(ta) = self.textareas.get(note) {
                let (row, col) = ta.cursor();
                if let Some(line) = ta.lines().get(row) {
                    let mut current_idx = 0;
                    while let Some(start) = line[current_idx..].find("[[") {
                        let start_idx = current_idx + start;
                        if let Some(end) = line[start_idx..].find("]]") {
                            let end_idx = start_idx + end + 2;
                            if col >= start_idx && col <= end_idx {
                                let link = &line[start_idx + 2..end_idx - 2];
                                return Some(link.to_string());
                            }
                            current_idx = end_idx;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_link_at_cursor() {
        let mut editor = Editor::new();
        editor.set_content("test.md", "Check out [[Project Ideas]] for details.");

        // Place cursor inside the brackets
        if let Some(ref note) = editor.current_note {
            if let Some(ta) = editor.textareas.get_mut(note) {
                ta.move_cursor(tui_textarea::CursorMove::WordForward);
                ta.move_cursor(tui_textarea::CursorMove::WordForward);
            }
        }

        let link = editor.extract_wiki_link_at_cursor();
        assert_eq!(link, Some("Project Ideas".to_string()));
    }

    #[test]
    fn test_editor_stats_and_save_state() {
        let mut editor = Editor::new();
        assert_eq!(editor.current_note, None);
        assert_eq!(editor.word_count(), 0);
        assert_eq!(editor.char_count(), 0);
        assert!(!editor.has_unsaved_changes());

        editor.set_content("note.md", "Hello world!");
        assert_eq!(editor.current_note, Some("note.md".to_string()));
        assert_eq!(editor.word_count(), 2);
        assert_eq!(editor.char_count(), 12);
        assert!(!editor.has_unsaved_changes());

        // Reload with different content simulates edit
        if let Some(ref note) = editor.current_note {
            if let Some(orig) = editor.original_content.get_mut(note) {
                *orig = "Different initial".to_string();
            }
        }
        assert!(editor.has_unsaved_changes());

        editor.mark_saved();
        assert!(!editor.has_unsaved_changes());

        // Setting empty title clears current_note
        editor.set_content("", "");
        assert_eq!(editor.current_note, None);
    }
}

