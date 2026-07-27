use ratatui::crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub struct SearchBar<'a> {
    pub textarea: TextArea<'a>,
}

impl Default for SearchBar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchBar<'_> {
    #[must_use]
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 🔍 Search / Create ");
        textarea.set_block(block);

        Self { textarea }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let mut ta = self.textarea.clone();

        if is_focused {
            ta.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
            ta.set_style(Style::default().fg(Color::Yellow)); // Yellow for search as per style guide
            ta.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔍 Search / Create ")
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        } else {
            ta.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
            ta.set_style(Style::default().fg(Color::Gray));
            ta.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔍 Search / Create ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        }

        frame.render_widget(&ta, area);
    }

    #[must_use]
    pub fn query(&self) -> String {
        self.textarea.lines().first().cloned().unwrap_or_default()
    }
}
