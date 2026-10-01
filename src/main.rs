use ferronote::{app::App, config::Config, event::EventHandler, export, tui::Tui};
use ferronote_store::NoteStore;

use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional custom config directory
    #[arg(short = 'c', long, env = "FERRONOTE_CONFIG_DIR")]
    config_dir: Option<PathBuf>,

    /// Optional custom notes directory
    #[arg(short, long, env = "FERRONOTE_NOTES_DIR")]
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

    /// Export the vault to a .zip archive, or a note (see --note) to an HTML file
    #[arg(short, long)]
    export: Option<PathBuf>,

    /// Note to export as HTML with --export: a title or filename
    #[arg(short, long, requires = "export", value_name = "NAME")]
    note: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup error handling
    color_eyre::install()?;

    // 2. Parse CLI args
    let args = Args::parse();

    // 3. Load config and initialize NoteStore
    let mut config = Config::load(args.config_dir)?;
    if let Some(custom_dir) = args.dir {
        config.notes_dir = custom_dir;
    }

    let mut note_store = NoteStore::new(config.notes_dir.clone())?;

    if let Some(export_path) = args.export {
        println!(
            "{}",
            export::export_for_cli(&note_store, &export_path, args.note.as_deref())?
        );
        return Ok(());
    }

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
    let tui = Tui::init(config.mouse_capture)?;

    // 5. Create App and EventHandler
    let events = EventHandler::new(
        std::time::Duration::from_millis(50),
        Some(config.notes_dir.clone()),
    );
    let mut app = App::new(note_store, config);

    // 6. Run the main loop
    let result = app.run(tui, events).await;

    // 6. Restore terminal before exiting
    Tui::restore()?;

    // Return the result of the app run loop
    result
}
