use crate::components::{Component, DrawableComponent, EventState};
use crate::event::Event;
use crate::queue::Queue;
use ratatui::crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Row, Table},
};

/// A popup component that displays help / keyboard shortcuts.
#[derive(Debug, Clone)]
pub struct HelpPopup {
    pub queue: Queue,
    pub scroll_offset: usize,
}

impl HelpPopup {
    #[must_use]
    pub fn new(queue: Queue) -> Self {
        Self {
            queue,
            scroll_offset: 0,
        }
    }
}

impl Component for HelpPopup {
    fn event(&mut self, ev: &Event) -> color_eyre::Result<EventState> {
        if let Event::Key(key) = ev {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') => {
                    // We emit ToggleHelp so the App knows to pop the help modal
                    self.queue.push(crate::action::Action::ToggleHelp);
                    return Ok(EventState::Consumed);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                    return Ok(EventState::Consumed);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    return Ok(EventState::Consumed);
                }
                _ => {
                    // Block other keys while help is open
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
}

impl DrawableComponent for HelpPopup {
    fn draw(&self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let keybindings_data = [
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
        ];

        let width = std::cmp::min(74, area.width.saturating_mul(80) / 100);
        let height = std::cmp::min(keybindings_data.len() as u16 + 2, area.height.saturating_mul(80) / 100);
        let x = area.width.saturating_sub(width) / 2;
        let y = area.height.saturating_sub(height) / 2;
        let popup_area = ratatui::layout::Rect::new(x, y, width, height);
        frame.render_widget(Clear, popup_area);

        let help_block = Block::default()
            .title(ratatui::text::Line::from(" Help / Keybindings ").alignment(ratatui::layout::Alignment::Center))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let rows: Vec<Row> = keybindings_data
            .iter()
            .map(|(k, d)| {
                Row::new(vec!["", *k, *d]).style(Style::default().fg(Color::White))
            })
            .collect();

        // Calculate max scroll
        let max_scroll = rows.len().saturating_sub(popup_area.height.saturating_sub(2) as usize);
        let actual_scroll = self.scroll_offset.min(max_scroll);

        let visible_rows = rows.into_iter().skip(actual_scroll);

        let table = Table::new(
            visible_rows,
            [Constraint::Length(2), Constraint::Length(18), Constraint::Min(40)],
        )
        .block(help_block)
        .column_spacing(2);

        frame.render_widget(table, popup_area);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_help_popup_events() {
        let queue = Queue::new();
        let mut help = HelpPopup::new(queue.clone());
        assert_eq!(help.scroll_offset, 0);

        // Scroll down
        let ev_down = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        let res1 = help.event(&ev_down).unwrap();
        assert_eq!(res1, EventState::Consumed);
        assert_eq!(help.scroll_offset, 1);

        // Scroll up
        let ev_up = Event::Key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        let res2 = help.event(&ev_up).unwrap();
        assert_eq!(res2, EventState::Consumed);
        assert_eq!(help.scroll_offset, 0);

        // Esc emits ToggleHelp action
        let ev_esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        let res_esc = help.event(&ev_esc).unwrap();
        assert_eq!(res_esc, EventState::Consumed);
        assert_eq!(queue.pop(), Some(crate::action::Action::ToggleHelp));
    }

    #[test]
    fn test_help_popup_draw() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let queue = Queue::new();
        let mut help = HelpPopup::new(queue);
        help.scroll_offset = 2;

        terminal.draw(|f| {
            let area = f.area();
            help.draw(f, area).unwrap();
        }).unwrap();
    }
}


