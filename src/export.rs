//! Backs the "export from UI" prompt: picks a default target path and runs
//! the note-to-HTML or vault-to-zip export the user's chosen path asks for.

use color_eyre::{Result, eyre::bail};
use ferronote_store::NoteStore;
use std::path::{Path, PathBuf};

/// Outcome of the last export attempt, shown inside the export prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportFeedback {
    /// The export failed; the prompt stays open so the path can be fixed.
    Error(String),
    /// The export succeeded; any key closes the prompt.
    Done(String),
}

/// Folder the export prompt suggests: Downloads, else home, else the
/// current directory.
#[must_use]
pub fn default_dir() -> PathBuf {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Suggests a path in `dir` that does not exist yet: `<note>.html` for the
/// selected note, or a dated vault archive when no note is selected.
#[must_use]
pub fn default_target(dir: &Path, note: Option<&str>) -> PathBuf {
    let (stem, ext) = match note {
        Some(filename) => (
            filename.strip_suffix(".md").unwrap_or(filename).to_string(),
            "html",
        ),
        None => (
            format!("ferronote-{}", chrono::Local::now().format("%Y%m%d")),
            "zip",
        ),
    };
    let first = dir.join(format!("{stem}.{ext}"));
    if !first.exists() {
        return first;
    }
    (1..10_000)
        .map(|n| dir.join(format!("{stem} {n}.{ext}")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}

/// Exports to the path typed into the prompt. A `.zip` path archives the
/// whole vault; any other path writes `note` as HTML. Never overwrites an
/// existing file. Returns a one-line summary on success.
///
/// # Errors
/// Returns a user-facing message when the path is unusable, there is no note
/// to export, or writing fails.
pub fn export(store: &NoteStore, note: Option<&str>, input: &str) -> Result<String> {
    let input = input.trim();
    if input.is_empty() {
        bail!("Enter a file path to export to");
    }
    let path = expand_tilde(input);
    if path.is_dir() {
        bail!("{} is a folder; add a file name", path.display());
    }
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
        && !parent.is_dir()
    {
        bail!("Folder {} does not exist", parent.display());
    }

    if is_zip(&path) {
        let count = store.export_vault_to_zip(&path)?;
        return Ok(format!("Exported {count} note(s) to {}", path.display()));
    }

    let Some(note) = note else {
        bail!("No note selected; use a .zip path to export the whole vault");
    };
    let written = store.export_note_to_html(note, &path)?;
    let title = note.strip_suffix(".md").unwrap_or(note);
    Ok(format!("Exported '{title}' to {}", written.display()))
}

/// Command-line export (`--export <PATH> [--note <NAME>]`). A `.zip` path
/// archives the whole vault; any other path writes the note named `note`
/// (a title or filename) as HTML. Unlike the in-app prompt this overwrites an
/// existing file, as scripts expect. Returns a one-line summary on success.
///
/// # Errors
/// Returns a user-facing message when `--note` is missing for an HTML export,
/// is combined with a `.zip` path, names no note, or writing fails.
pub fn export_for_cli(store: &NoteStore, path: &Path, note: Option<&str>) -> Result<String> {
    if is_zip(path) {
        if note.is_some() {
            bail!("--note cannot be used with a .zip export; a .zip exports the whole vault");
        }
        let count = store.export_vault_to_zip(path)?;
        return Ok(format!("Exported {count} note(s) to zip archive: {path:?}"));
    }

    let Some(name) = note else {
        bail!(
            "Exporting HTML needs a note: pass --note <NAME>, or use a .zip path for the whole vault"
        );
    };
    let Some(filename) = store.resolve_note(name) else {
        bail!("Note '{name}' not found");
    };
    let written = store.export_note_to_html(&filename, path)?;
    Ok(format!("Exported '{filename}' to HTML: {written:?}"))
}

fn is_zip(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
}

/// Expands a leading `~` or `~/` to the user's home directory.
fn expand_tilde(input: &str) -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        if input == "~" {
            return home;
        }
        if let Some(rest) = input.strip_prefix("~/") {
            return home.join(rest);
        }
    }
    PathBuf::from(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with_note() -> (NoteStore, tempfile::TempDir, tempfile::TempDir) {
        let notes = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        let mut store = NoteStore::new(notes.path().to_path_buf()).unwrap();
        store
            .create_note_with_content("Plan", "# Plan\n\nship it")
            .unwrap();
        (store, notes, out)
    }

    #[test]
    fn test_default_target_for_note_and_vault() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            default_target(dir.path(), Some("My Note.md")),
            dir.path().join("My Note.html")
        );
        let vault = default_target(dir.path(), None);
        assert_eq!(vault.extension().and_then(|e| e.to_str()), Some("zip"));
        assert!(
            vault
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ferronote-"))
        );
    }

    #[test]
    fn test_default_target_skips_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Note.html"), "x").unwrap();
        std::fs::write(dir.path().join("Note 1.html"), "x").unwrap();
        assert_eq!(
            default_target(dir.path(), Some("Note.md")),
            dir.path().join("Note 2.html")
        );
    }

    #[test]
    fn test_export_note_as_html() {
        let (store, _notes, out) = store_with_note();
        let target = out.path().join("plan.html");

        let msg = export(&store, Some("Plan.md"), target.to_str().unwrap()).unwrap();

        assert!(msg.contains("Exported 'Plan'"));
        let html = std::fs::read_to_string(&target).unwrap();
        assert!(html.contains("<h1>Plan</h1>"));
    }

    #[test]
    fn test_export_vault_as_zip_regardless_of_selection() {
        let (store, _notes, out) = store_with_note();
        let target = out.path().join("vault.ZIP");

        let msg = export(&store, None, target.to_str().unwrap()).unwrap();

        assert!(msg.starts_with("Exported 3 note(s)"));
        assert!(target.is_file());
    }

    #[test]
    fn test_export_rejects_bad_targets() {
        let (store, _notes, out) = store_with_note();
        let existing = out.path().join("taken.html");
        std::fs::write(&existing, "keep me").unwrap();

        let err = |input: &str, note| export(&store, note, input).unwrap_err().to_string();

        assert!(err("  ", Some("Plan.md")).contains("Enter a file path"));
        assert!(err(out.path().to_str().unwrap(), Some("Plan.md")).contains("is a folder"));
        assert!(err(existing.to_str().unwrap(), Some("Plan.md")).contains("already exists"));
        assert_eq!(std::fs::read_to_string(&existing).unwrap(), "keep me");
        assert!(
            err(
                out.path().join("missing/x.html").to_str().unwrap(),
                Some("Plan.md")
            )
            .contains("does not exist")
        );
        assert!(
            err(out.path().join("x.html").to_str().unwrap(), None).contains("No note selected")
        );
        assert!(!out.path().join("x.html").exists());
    }

    #[test]
    fn test_export_for_cli_html_exports_the_named_note() {
        let (mut store, _notes, out) = store_with_note();
        store
            .create_note_with_content("Other", "# Other\n\nnot this one")
            .unwrap();
        let target = out.path().join("plan.html");

        let msg = export_for_cli(&store, &target, Some("plan")).unwrap();

        assert!(msg.contains("Exported 'Plan.md' to HTML"));
        let html = std::fs::read_to_string(&target).unwrap();
        assert!(html.contains("<h1>Plan</h1>"));
        assert!(!html.contains("not this one"));

        // Overwrites, unlike the in-app prompt.
        export_for_cli(&store, &target, Some("Other.md")).unwrap();
        assert!(
            std::fs::read_to_string(&target)
                .unwrap()
                .contains("not this one")
        );
    }

    #[test]
    fn test_export_for_cli_zip_exports_whole_vault() {
        let (store, _notes, out) = store_with_note();
        let target = out.path().join("vault.zip");

        let msg = export_for_cli(&store, &target, None).unwrap();

        assert!(msg.starts_with("Exported 3 note(s) to zip archive"));
        assert!(target.is_file());
    }

    #[test]
    fn test_export_for_cli_rejects_bad_combinations() {
        let (store, _notes, out) = store_with_note();
        let html = out.path().join("x.html");
        let zip = out.path().join("x.zip");

        let err = |path: &Path, note| export_for_cli(&store, path, note).unwrap_err().to_string();

        assert!(err(&html, None).contains("--note <NAME>"));
        assert!(err(&html, Some("Nope")).contains("Note 'Nope' not found"));
        assert!(err(&zip, Some("Plan")).contains("cannot be used with a .zip"));
        assert!(!html.exists());
        assert!(!zip.exists());
    }

    #[test]
    fn test_expand_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/a/b.zip"), home.join("a/b.zip"));
        assert_eq!(expand_tilde("~other/b.zip"), PathBuf::from("~other/b.zip"));
        assert_eq!(expand_tilde("rel/b.zip"), PathBuf::from("rel/b.zip"));
    }
}
