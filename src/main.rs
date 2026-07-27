pub mod action;
pub mod app;
pub mod components;
pub mod event;
pub mod tui;

use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

use crate::{app::App, event::EventHandler, tui::Tui};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional custom notes directory
    #[arg(short, long)]
    dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup error handling
    color_eyre::install()?;

    // 2. Parse CLI args
    let _args = Args::parse(); // Unused in Phase 0, but scaffolded

    // 3. Initialize terminal (enters alternate screen, raw mode, sets panic hook)
    let tui = Tui::init()?;

    // 4. Create App and EventHandler
    let mut app = App::new();
    let events = EventHandler::new(std::time::Duration::from_millis(250));

    // 5. Run the main loop
    let result = app.run(tui, events).await;

    // 6. Restore terminal before exiting
    Tui::restore()?;

    // Return the result of the app run loop
    result
}
