use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{
    action::Action,
    components::{editor::Editor, note_list::NoteList, search_bar::SearchBar},
    event::{Event, EventHandler},
    focus::Focus,
    note_store::NoteStore,
    tui::Tui,
};

#[derive(Debug)]
pub struct App<'a> {
    pub should_quit: bool,
    pub note_store: NoteStore,
    pub focus: Focus,
    pub search_bar: SearchBar<'a>,
    pub note_list: NoteList,
    pub editor: Editor<'a>,
}

impl App<'_> {
    #[must_use]
    pub fn new(note_store: NoteStore) -> Self {
        Self {
            should_quit: false,
            note_store,
            focus: Focus::default(),
            search_bar: SearchBar::new(),
            note_list: NoteList::new(),
            editor: Editor::new(),
        }
    }

    /// # Errors
    /// Returns an error if drawing to the terminal fails or the event stream closes.
    pub async fn run(&mut self, mut tui: Tui, mut events: EventHandler) -> Result<()> {
        while !self.should_quit {
            tui.draw(|frame| self.draw(frame))?;

            let event = events.next().await?;
            let action = self.handle_event(event);

            if let Some(action) = action {
                self.update(action);
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Tick => Some(Action::Tick),
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Mouse(_) => None,
            Event::Resize(w, h) => Some(Action::Resize(w, h)),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('q')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                return Some(Action::Quit);
            }
            KeyCode::Tab => {
                self.focus = self.focus.next();
                return None;
            }
            KeyCode::Esc => {
                self.focus = Focus::SearchBar;
                return None;
            }
            _ => {}
        }

        match self.focus {
            Focus::SearchBar => {
                self.search_bar.handle_key(key);
            }
            Focus::NoteList => {
                if key.code == KeyCode::Down {
                    self.note_list.next();
                } else if key.code == KeyCode::Up {
                    self.note_list.previous();
                }
            }
            Focus::Editor => {
                self.editor.handle_key(key);
            }
        }

        None
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Tick | Action::Render | Action::Resize(_, _) => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(1),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area());

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Note list
                Constraint::Percentage(70), // Editor
            ])
            .split(main_layout[1]);

        // Render components
        self.search_bar
            .draw(frame, main_layout[0], self.focus == Focus::SearchBar);
        self.note_list
            .draw(frame, content_layout[0], self.focus == Focus::NoteList);
        self.editor
            .draw(frame, content_layout[1], self.focus == Focus::Editor);

        // Render status bar
        let status_text = Line::from(vec![
            Span::styled(" [Tab/Esc] ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch Focus  |"),
            Span::styled(" [Ctrl+N] ", Style::default().fg(Color::Yellow)),
            Span::raw("New Note  |"),
            Span::styled(" [Ctrl+D] ", Style::default().fg(Color::Yellow)),
            Span::raw("Delete  |"),
            Span::styled(" [Ctrl+Q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]);
        let status_bar = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        frame.render_widget(status_bar, main_layout[2]);
    }
}
