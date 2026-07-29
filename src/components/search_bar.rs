use ratatui::crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

/// Unified search bar and note creation input component.
#[derive(Debug, Clone)]
pub struct SearchBar<'a> {
    /// Inner text area holding search query input.
    pub textarea: TextArea<'a>,
}

impl Default for SearchBar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchBar<'_> {
    /// Creates a new `SearchBar` instance initialized with border box styling.
    #[must_use]
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 🔍 Search / Create ");
        textarea.set_block(block);

        Self { textarea }
    }

    /// Forwards keyboard events to the search text area.
    pub fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
    }

    /// Clears the search bar input.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        area: Rect,
        is_focused: bool,
        theme: &crate::theme::ThemePalette,
    ) {
        let mut ta = self.textarea.clone();

        if is_focused {
            ta.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
            ta.set_style(Style::default().fg(theme.search_fg));
            ta.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔍 Search / Create ")
                    .border_style(Style::default().fg(theme.border_active)),
            );
        } else {
            ta.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
            ta.set_style(Style::default().fg(Color::Gray));
            ta.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔍 Search / Create ")
                    .border_style(Style::default().fg(theme.border_inactive)),
            );
        }

        frame.render_widget(&ta, area);
    }

    #[must_use]
    pub fn query(&self) -> String {
        self.textarea.lines().first().cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_search_bar_input_and_query() {
        let mut search_bar = SearchBar::new();
        assert_eq!(search_bar.query(), "");

        search_bar.handle_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));

        assert_eq!(search_bar.query(), "rust");

        search_bar.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(search_bar.query(), "rus");
    }

    #[test]
    fn test_search_bar_clear() {
        let mut search_bar = SearchBar::new();
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        assert_eq!(search_bar.query(), "foo");

        search_bar.clear();
        assert_eq!(search_bar.query(), "");
    }
}
