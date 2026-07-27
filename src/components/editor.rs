use ratatui::crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub struct Editor<'a> {
    pub textarea: TextArea<'a>,
    pub current_note: Option<String>,
}

impl Default for Editor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor<'_> {
    #[must_use]
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Editor ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        Self {
            textarea,
            current_note: None,
        }
    }

    pub fn set_content(&mut self, title: &str, content: &str) {
        self.current_note = Some(title.to_string());

        let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
        self.textarea = TextArea::new(lines);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.textarea.input(key);
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let mut ta = self.textarea.clone();

        let title = if let Some(ref note) = self.current_note {
            format!(" {} ", note.strip_suffix(".md").unwrap_or(note))
        } else {
            " Editor ".to_string()
        };

        if is_focused {
            ta.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
            ta.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Blue)),
            );
        } else {
            // Hide cursor when not focused
            ta.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
            ta.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        }

        frame.render_widget(&ta, area);
    }

    #[must_use]
    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }
}
