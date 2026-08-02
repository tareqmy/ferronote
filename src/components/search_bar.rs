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
    /// Event queue for emitting app actions.
    pub queue: crate::queue::Queue,
}

// Default removed due to queue requirement

impl SearchBar<'_> {
    /// Creates a new `SearchBar` instance initialized with border box styling.
    #[must_use]
    pub fn new(queue: crate::queue::Queue) -> Self {
        let mut textarea = TextArea::default();
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 🔍 Search / Create ");
        textarea.set_block(block);

        Self { textarea, queue }
    }

    /// Forwards keyboard events to the search text area.
    pub fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
    }

    /// Clears the search bar input.
    pub fn clear(&mut self) {
        *self = Self::new(self.queue.clone());
    }
}

impl crate::components::Component for SearchBar<'_> {
    fn event(&mut self, ev: &crate::event::Event) -> color_eyre::Result<crate::components::EventState> {
        if let crate::event::Event::Key(key) = ev {
            self.handle_key(*key);
            // SearchBar consumes all keys when focused (usually)
            return Ok(crate::components::EventState::Consumed);
        }
        Ok(crate::components::EventState::NotConsumed)
    }
}

impl SearchBar<'_> {

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
        let queue = crate::queue::Queue::new();
        let mut search_bar = SearchBar::new(queue);
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
        let queue = crate::queue::Queue::new();
        let mut search_bar = SearchBar::new(queue);
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        assert_eq!(search_bar.query(), "foo");

        search_bar.clear();
        assert_eq!(search_bar.query(), "");
    }

    #[test]
    fn test_search_bar_component_event() {
        use crate::components::Component;
        let queue = crate::queue::Queue::new();
        let mut search_bar = SearchBar::new(queue);

        let ev = crate::event::Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        let res = search_bar.event(&ev).unwrap();
        assert_eq!(res, crate::components::EventState::Consumed);
        assert_eq!(search_bar.query(), "a");
    }

    #[test]
    fn test_search_bar_draw() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        let queue = crate::queue::Queue::new();
        let mut search_bar = SearchBar::new(queue);
        search_bar.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
        let theme = crate::theme::ThemePalette::from_name("default");

        terminal.draw(|f| {
            let area = f.area();
            search_bar.draw(f, area, true, &theme);
        }).unwrap();

        terminal.draw(|f| {
            let area = f.area();
            search_bar.draw(f, area, false, &theme);
        }).unwrap();
    }
}
