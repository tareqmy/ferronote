use ratatui::crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;
use std::collections::HashMap;
use std::time::Instant;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct Editor<'a> {
    pub textareas: HashMap<String, TextArea<'a>>,
    pub current_note: Option<String>,
    pub last_edit_time: Option<Instant>,
    pub original_content: HashMap<String, String>,
}

impl Default for Editor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor<'_> {
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
            self.original_content.insert(title.to_string(), content.to_string());
        } else {
            // Check if we need to reload due to external change (e.g. content doesn't match and no unsaved changes)
            if !self.has_unsaved_changes() && self.content() != content {
                let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
                let ta = TextArea::new(lines);
                self.textareas.insert(title.to_string(), ta);
                self.original_content.insert(title.to_string(), content.to_string());
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(ref note) = self.current_note {
            if let Some(ta) = self.textareas.get_mut(note) {
                let modified = if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('z') {
                    ta.undo()
                } else if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('y') {
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

    pub fn draw(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let title = if let Some(ref note) = self.current_note {
            format!(" {} ", note.strip_suffix(".md").unwrap_or(note))
        } else {
            " Editor ".to_string()
        };

        let mut block = Block::default().borders(Borders::ALL).title(title.clone());
        if is_focused {
            block = block.border_style(Style::default().fg(Color::Blue));
        } else {
            block = block.border_style(Style::default().fg(Color::DarkGray));
        }

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
            self.original_content.insert(note.to_string(), self.content());
            self.last_edit_time = None;
        }
    }
}
