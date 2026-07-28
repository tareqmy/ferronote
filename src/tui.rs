use color_eyre::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::DefaultTerminal;
use std::io::stdout;

/// A thin wrapper around `ratatui::DefaultTerminal`
pub struct Tui {
    terminal: DefaultTerminal,
}

impl Tui {
    /// Initializes the terminal, entering alternate screen, raw mode, and enabling mouse capture.
    /// # Errors
    /// Returns an error if terminal initialization fails.
    pub fn init() -> Result<Self> {
        let terminal = ratatui::init();
        let _ = execute!(stdout(), EnableMouseCapture);
        Ok(Self { terminal })
    }

    /// Restores the terminal to its normal state.
    /// # Errors
    /// Returns an error if terminal restoration fails.
    pub fn restore() -> Result<()> {
        let _ = execute!(stdout(), DisableMouseCapture);
        ratatui::restore();
        Ok(())
    }
}

impl std::ops::Deref for Tui {
    type Target = DefaultTerminal;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl std::ops::DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}
