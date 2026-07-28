use ferronote::{app::App, config::Config, event::EventHandler, note_store::NoteStore, tui::Tui};

use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional custom notes directory
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Path to file, directory, or .zip archive to import
    #[arg(short, long)]
    import: Option<PathBuf>,

    /// List trashed notes
    #[arg(long)]
    trash: bool,

    /// Restore trashed note by filename
    #[arg(long)]
    restore: Option<String>,
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

    let mut note_store = NoteStore::new(config.notes_dir.clone())?;

    if args.trash {
        let trash_list = note_store.list_trash()?;
        if trash_list.is_empty() {
            println!("Trash is empty.");
        } else {
            println!("Trashed Notes ({} total):", trash_list.len());
            for (filename, original) in trash_list {
                println!("  - {} (Original: {})", filename, original);
            }
        }
        return Ok(());
    }

    if let Some(trash_filename) = args.restore {
        let restored = note_store.restore_note(&trash_filename)?;
        println!("Successfully restored note as '{}'", restored);
        return Ok(());
    }

    if let Some(import_path) = args.import {
        let count = note_store.import_path(&import_path)?;
        println!(
            "Successfully imported {} note(s) into {:?}",
            count, config.notes_dir
        );
        return Ok(());
    }

    // 4. Initialize terminal (enters alternate screen, raw mode, sets panic hook)
    let tui = Tui::init()?;

    // 5. Create App and EventHandler
    let mut app = App::new(note_store);
    let events = EventHandler::new(std::time::Duration::from_millis(50), Some(config.notes_dir));

    // 6. Run the main loop
    let result = app.run(tui, events).await;

    // 6. Restore terminal before exiting
    Tui::restore()?;

    // Return the result of the app run loop
    result
}
