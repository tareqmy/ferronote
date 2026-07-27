pub mod action;
pub mod app;
pub mod components;
pub mod config;
pub mod event;
pub mod note_store;
pub mod tui;

use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

use crate::{app::App, config::Config, event::EventHandler, note_store::NoteStore, tui::Tui};

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
    let args = Args::parse();

    // 3. Load config and initialize NoteStore
    let mut config = Config::load()?;
    if let Some(custom_dir) = args.dir {
        config.notes_dir = custom_dir;
    }

    let note_store = NoteStore::new(config.notes_dir)?;

    // 4. Initialize terminal (enters alternate screen, raw mode, sets panic hook)
    let tui = Tui::init()?;

    // 5. Create App and EventHandler
    let mut app = App::new(note_store);
    let events = EventHandler::new(std::time::Duration::from_millis(250));

    // 6. Run the main loop
    let result = app.run(tui, events).await;

    // 6. Restore terminal before exiting
    Tui::restore()?;

    // Return the result of the app run loop
    result
}
