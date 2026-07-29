use clap::Parser;
use color_eyre::Result;
use ferronote::{app::App, config::Config, event::EventHandler, note_store::NoteStore, tui::Tui};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fnt", version, about = "Ferronote terminal note-taking app shortcut", long_about = None)]
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

    /// Export notes to zip archive or HTML file
    #[arg(short, long)]
    export: Option<PathBuf>,
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

    if let Some(export_path) = args.export {
        let ext = export_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext == "zip" {
            let count = note_store.export_vault_to_zip(&export_path)?;
            println!(
                "Exported {} note(s) to zip archive: {:?}",
                count, export_path
            );
        } else {
            let first_note = note_store
                .filenames()
                .first()
                .cloned()
                .ok_or_else(|| color_eyre::eyre::eyre!("No notes found to export"))?;
            let html_path = note_store.export_note_to_html(&first_note, &export_path)?;
            println!("Exported '{}' to HTML: {:?}", first_note, html_path);
        }
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

    // 4. Initialize terminal
    let tui = Tui::init()?;

    // 5. Create App and EventHandler
    let mut app = App::new(note_store);
    let events = EventHandler::new(std::time::Duration::from_millis(50), Some(config.notes_dir));

    // 6. Run the main loop
    let result = app.run(tui, events).await;

    // 7. Restore terminal
    Tui::restore()?;

    result
}
