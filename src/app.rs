use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{
    action::Action,
    event::{Event, EventHandler},
    tui::Tui,
};

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Errors
    /// Returns an error if drawing to the terminal fails or the event stream closes.
    pub async fn run(&mut self, mut tui: Tui, mut events: EventHandler) -> Result<()> {
        while !self.should_quit {
            tui.draw(|frame| self.draw(frame))?;

            let event = events.next().await?;
            let action = Self::handle_event(event);

            if let Some(action) = action {
                self.update(action);
            }
        }

        Ok(())
    }

    fn handle_event(event: Event) -> Option<Action> {
        match event {
            Event::Tick => Some(Action::Tick),
            Event::Key(key_event) => Self::handle_key(key_event),
            Event::Mouse(_) => None,
            Event::Resize(w, h) => Some(Action::Resize(w, h)),
        }
    }

    fn handle_key(key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Char('q')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                Some(Action::Quit)
            }
            _ => None,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Tick | Action::Render | Action::Resize(_, _) => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let size = frame.area();

        let text = "🗒️ Ferronote — Notes at the speed of thought";
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Ferronote ")
                    .title_alignment(Alignment::Center),
            )
            .style(Style::default().fg(Color::White));

        // Render centered text box
        // Handle case where terminal might be too small
        let x = size.x.saturating_add(size.width / 4);
        let y = size.y.saturating_add(size.height / 2).saturating_sub(2);
        let width = (size.width / 2).max(10);
        let height = 3;

        let inner_area = Rect::new(x, y, width.min(size.width), height.min(size.height));

        frame.render_widget(paragraph, inner_area);
    }
}
