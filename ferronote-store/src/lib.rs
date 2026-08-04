use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteMetadata {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    #[serde(default)]
    pub is_pinned: bool,
}

#[derive(Debug)]
pub struct NoteStore {
    notes_dir: PathBuf,
    metadata_path: PathBuf,
    metadata: HashMap<String, NoteMetadata>,
}

impl NoteStore {
    /// # Errors
    /// Returns an error if directory creation fails.
    pub fn new(notes_dir: PathBuf) -> Result<Self> {
        if !notes_dir.exists() {
            std::fs::create_dir_all(&notes_dir)?;
        }

        // Create trash directory directly in notes_dir
        let trash_dir = notes_dir.join("trash");
        if !trash_dir.exists() {
            std::fs::create_dir_all(&trash_dir)?;
        }

        let metadata_path = notes_dir.join("metadata.json");

        let mut store = Self {
            notes_dir,
            metadata_path,
            metadata: HashMap::new(),
        };

        store.load_metadata()?;
        store.scan_directory()?;

        let _ = store.create_default_welcome_note();
        let _ = store.create_default_lorem_ipsum_note();

        Ok(store)
    }

    /// Gets the absolute path for a note filename.
    pub fn get_note_path(&self, filename: &str) -> PathBuf {
        self.notes_dir.join(filename)
    }

    /// Creates or updates the default welcome note with the latest application guide content.
    /// # Errors
    /// Returns an error if saving the note fails.
    pub fn create_default_welcome_note(&mut self) -> Result<String> {
        let title = "Welcome to Ferronote";
        let filename = format!("{title}.md");
        let path = self.notes_dir.join(&filename);

        if !path.exists() {
            let content = r#"# Welcome to Ferronote

Ferronote is a blazing-fast terminal note-taking app inspired by Notational Velocity.

## 🚀 Key Concept: "Search IS Create"
You don't need a separate "New Note" button!
1. Type a note title or search query in the top search bar.
2. If a note with that title exists, it will filter and open it.
3. If no matching note exists, press Enter to instantly create and edit a note with that title.

## ⌨️ Essential Keybindings
- / or Ctrl+L: Focus search bar.
- Esc: Clear search bar / Close overlay.
- Ctrl+N: Start a new note.
- Tab: Cycle focus between Search Bar, Note List, and Editor.
- Up / Down: Navigate notes in the list.
- PgUp / PgDn: Scroll note list page by page.
- Home / End: Jump to top or bottom of notes list.
- Ctrl+S: Force save current note.
- Ctrl+D: Soft delete selected note (prompts for y/n, moves file to trash).
- Ctrl+K: Toggle Pin / Bookmark on selected note (or 'p' in Note List).
- Ctrl+B: Toggle Notes List panel visibility.
- Ctrl+Z / Ctrl+Y: Undo / Redo inside the editor.
- Ctrl+O: Open active note in external advanced editor (configure in Ctrl+P settings).
- Ctrl+P: Toggle Interactive Settings Overlay.
- Ctrl+V: Toggle About Overlay.
- ?: Toggle Help Overlay.
- Ctrl+Q: Quit Ferronote.

## ⚡ Vim Keybindings (Experimental)
- h, j, k, l: Move cursor left, down, up, right (View mode).
- w, b, W, B: Move word forward/backward, and by whitespace (View mode).
- e, ge: Move to end of word forward/backward (View mode).
- 0, ^, $: Move to beginning, first non-blank, and end of line (View mode).
- +, -: Move to first non-blank char of next/prev line (View mode).
- gg, G: Move to beginning, end of file (View mode).
- %: Jump to matching brace/bracket (View mode).
- dd, dNd: Delete current line, or N lines (View mode).
- yy, yNy: Yank (copy) current line, or N lines (View mode).
- p, P: Paste after, before cursor (View mode).
- u, Ctrl+r: Undo, Redo (View mode).
- /, n, N: Search forward (regex), next match, previous match (View mode).
- x: Delete character under cursor (View mode).
- Ctrl+W: Delete word before cursor (Edit mode).
- Ctrl+U: Delete to beginning of line (Edit mode).
- Ctrl+H: Delete character before cursor (Edit mode).

## 🏷️ Tags & Wiki-Links
- #tags: Add inline tags like #todo or #ideas anywhere in your note, then search #todo to filter instantly.
- [[Wiki-Links]]: Type [[Another Note]] inside a note, place your cursor on it, and press Enter to jump to or create the linked note! Check out [[Lorem Ipsum]] for an example.

Happy note taking!
"#;
            std::fs::write(&path, content)?;

            let now = Utc::now();
            self.metadata.insert(
                filename.clone(),
                NoteMetadata {
                    created_at: now,
                    modified_at: now,
                    is_pinned: false,
                },
            );
            self.save_metadata()?;
        }

        Ok(filename)
    }

    /// Creates or updates the default secondary note with Lorem Ipsum text.
    /// # Errors
    /// Returns an error if saving the note fails.
    pub fn create_default_lorem_ipsum_note(&mut self) -> Result<String> {
        let title = "Lorem Ipsum";
        let filename = format!("{title}.md");
        let path = self.notes_dir.join(&filename);

        if !path.exists() {
            let content = r#"# Lorem Ipsum

Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

---
💡 Referencing [[Welcome to Ferronote]]
💡 Press `Ctrl+O` to open this note in your advanced external editor!
"#;
            std::fs::write(&path, content)?;

            let now = Utc::now();
            self.metadata.insert(
                filename.clone(),
                NoteMetadata {
                    created_at: now,
                    modified_at: now,
                    is_pinned: false,
                },
            );
            self.save_metadata()?;
        }

        Ok(filename)
    }

    fn load_metadata(&mut self) -> Result<()> {
        if self.metadata_path.exists() {
            let content = std::fs::read_to_string(&self.metadata_path)?;
            if let Ok(metadata) = serde_json::from_str(&content) {
                self.metadata = metadata;
            }
        }
        Ok(())
    }

    fn save_metadata(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.metadata)?;
        let tmp_path = self.metadata_path.with_extension("tmp");
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(tmp_path, &self.metadata_path)?;
        Ok(())
    }

    /// # Errors
    /// Returns an error if reading the directory fails.
    pub fn scan_directory(&mut self) -> Result<()> {
        let mut found_files = Vec::new();

        for entry in std::fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str())
            {
                if ext == "md" {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        found_files.push(name.to_string());
                    }
                } else if ext == "tmp" && path.to_string_lossy().ends_with(".md.tmp") {
                    // Orphaned temporary file from a crash
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        let now = Utc::now();
        let mut modified = false;

        for file in &found_files {
            if !self.metadata.contains_key(file) {
                self.metadata.insert(
                    file.clone(),
                    NoteMetadata {
                        created_at: now,
                        modified_at: now,
                        is_pinned: false,
                    },
                );
                modified = true;
            }
        }

        // Remove metadata for files that no longer exist
        let keys_to_remove: Vec<String> = self
            .metadata
            .keys()
            .filter(|k| !found_files.contains(k))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.metadata.remove(&key);
            modified = true;
        }

        if modified {
            self.save_metadata()?;
        }

        Ok(())
    }

    /// # Errors
    /// Returns an error if the file cannot be read.
    pub fn load_note(&self, filename: &str) -> Result<String> {
        let path = self.notes_dir.join(filename);
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }

    /// # Errors
    /// Returns an error if import fails.
    pub fn import_path(&mut self, path: &std::path::Path) -> Result<usize> {
        if !path.exists() {
            color_eyre::eyre::bail!("Path does not exist: {:?}", path);
        }

        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "zip" {
                self.import_zip(path)
            } else if ext == "md" || ext == "txt" {
                self.import_file(path)?;
                Ok(1)
            } else {
                color_eyre::eyre::bail!(
                    "Unsupported file format. Please provide a .md, .txt, or .zip file."
                );
            }
        } else if path.is_dir() {
            self.import_directory(path)
        } else {
            color_eyre::eyre::bail!("Invalid path for import");
        }
    }

    /// # Errors
    /// Returns an error if file read or write fails.
    pub fn import_file(&mut self, file_path: &std::path::Path) -> Result<String> {
        let content = std::fs::read_to_string(file_path)?;
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Note");

        let safe_title = stem.replace(['/', '\\'], "-");
        let mut target_filename = format!("{safe_title}.md");
        let mut counter = 1;

        while self.notes_dir.join(&target_filename).exists() {
            target_filename = format!("{safe_title} {counter}.md");
            counter += 1;
        }

        let target_path = self.notes_dir.join(&target_filename);
        std::fs::write(&target_path, &content)?;

        let file_meta = std::fs::metadata(file_path)?;
        let modified_at: DateTime<Utc> =
            file_meta.modified().ok().map_or_else(Utc::now, Into::into);

        self.metadata.insert(
            target_filename.clone(),
            NoteMetadata {
                created_at: modified_at,
                modified_at,
                is_pinned: false,
            },
        );
        self.save_metadata()?;

        Ok(target_filename)
    }

    /// # Errors
    /// Returns an error if reading directory fails.
    pub fn import_directory(&mut self, dir_path: &std::path::Path) -> Result<usize> {
        let mut count = 0;
        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if (ext == "md" || ext == "txt") && self.import_file(&path).is_ok() {
                    count += 1;
                }
            } else if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.')
                    && name != "trash"
                    && let Ok(c) = self.import_directory(&path)
                {
                    count += c;
                }
            }
        }
        Ok(count)
    }

    /// # Errors
    /// Returns an error if reading or extracting zip fails.
    pub fn import_zip(&mut self, zip_path: &std::path::Path) -> Result<usize> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut count = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let ext = outpath.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !file.is_dir() && (ext == "md" || ext == "txt") {
                let mut content = String::new();
                use std::io::Read;
                if file.read_to_string(&mut content).is_ok() {
                    let stem = outpath
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Imported Note");

                    let safe_title = stem.replace(['/', '\\'], "-");
                    let mut target_filename = format!("{safe_title}.md");
                    let mut counter = 1;

                    while self.notes_dir.join(&target_filename).exists() {
                        target_filename = format!("{safe_title} {counter}.md");
                        counter += 1;
                    }

                    let target_path = self.notes_dir.join(&target_filename);
                    if std::fs::write(&target_path, &content).is_ok() {
                        let now = Utc::now();
                        self.metadata.insert(
                            target_filename,
                            NoteMetadata {
                                created_at: now,
                                modified_at: now,
                                is_pinned: false,
                            },
                        );
                        count += 1;
                    }
                }
            }
        }

        if count > 0 {
            self.save_metadata()?;
        }
        Ok(count)
    }

    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save_note(&mut self, filename: &str, content: &str) -> Result<()> {
        let path = self.notes_dir.join(filename);
        let tmp_path = path.with_extension("md.tmp");

        std::fs::write(&tmp_path, content)?;
        std::fs::rename(tmp_path, &path)?;

        if let Some(meta) = self.metadata.get_mut(filename) {
            meta.modified_at = Utc::now();
            self.save_metadata()?;
        }

        Ok(())
    }

    /// # Errors
    /// Returns an error if the file already exists or cannot be written.
    pub fn create_note(&mut self, title: &str) -> Result<String> {
        let safe_title = title.replace(['/', '\\'], "-");
        let filename = format!("{safe_title}.md");
        let path = self.notes_dir.join(&filename);

        if path.exists() {
            color_eyre::eyre::bail!("Note already exists");
        }

        let content = format!("# {title}\n\n");
        std::fs::write(&path, content)?;

        let now = Utc::now();
        self.metadata.insert(
            filename.clone(),
            NoteMetadata {
                created_at: now,
                modified_at: now,
                is_pinned: false,
            },
        );
        self.save_metadata()?;

        Ok(filename)
    }

    /// # Errors
    /// Returns an error if the file cannot be deleted.
    pub fn delete_note(&mut self, filename: &str) -> Result<()> {
        let file_path = self.notes_dir.join(filename);
        if file_path.exists() {
            let trash_dir = self.notes_dir.join("trash");

            // Generate a unique filename for the trash to avoid overwriting previously deleted notes with the same name
            let timestamp = chrono::Utc::now().timestamp();
            let trash_filename = format!("{}-{}", timestamp, filename);
            let trash_path = trash_dir.join(trash_filename);

            std::fs::rename(file_path, trash_path)?;
        }

        self.metadata.remove(filename);
        self.save_metadata()?;

        Ok(())
    }

    /// # Errors
    /// Returns an error if reading trash directory fails.
    pub fn list_trash(&self) -> Result<Vec<(String, String)>> {
        let trash_dir = self.notes_dir.join("trash");
        let mut list = Vec::new();
        if trash_dir.exists() {
            for entry in std::fs::read_dir(trash_dir)? {
                let entry = entry?;
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename.contains('-') {
                    let original_title = filename
                        .split_once('-')
                        .map(|x| x.1)
                        .unwrap_or(&filename)
                        .strip_suffix(".md")
                        .unwrap_or(&filename)
                        .to_string();
                    list.push((filename, original_title));
                }
            }
        }
        Ok(list)
    }

    /// # Errors
    /// Returns an error if moving file from trash fails.
    pub fn restore_note(&mut self, trash_filename: &str) -> Result<String> {
        let trash_dir = self.notes_dir.join("trash");
        let trash_path = trash_dir.join(trash_filename);
        if !trash_path.exists() {
            color_eyre::eyre::bail!("File not found in trash: {}", trash_filename);
        }

        let raw_title = trash_filename
            .split_once('-')
            .map(|x| x.1)
            .unwrap_or(trash_filename);

        let stem = raw_title.strip_suffix(".md").unwrap_or(raw_title);
        let safe_title = stem.replace(['/', '\\'], "-");
        let mut target_filename = format!("{safe_title}.md");
        let mut counter = 1;

        while self.notes_dir.join(&target_filename).exists() {
            target_filename = format!("{safe_title} {counter}.md");
            counter += 1;
        }

        let target_path = self.notes_dir.join(&target_filename);
        std::fs::rename(trash_path, target_path)?;

        let now = Utc::now();
        self.metadata.insert(
            target_filename.clone(),
            NoteMetadata {
                created_at: now,
                modified_at: now,
                is_pinned: false,
            },
        );
        self.save_metadata()?;

        Ok(target_filename)
    }

    /// # Errors
    /// Returns an error if removing trash files fails.
    pub fn purge_trash(&mut self) -> Result<usize> {
        let trash_dir = self.notes_dir.join("trash");
        let mut count = 0;
        if trash_dir.exists() {
            for entry in std::fs::read_dir(trash_dir)? {
                let entry = entry?;
                std::fs::remove_file(entry.path())?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// # Errors
    /// Returns an error if reading note or writing HTML file fails.
    pub fn export_note_to_html(
        &self,
        filename: &str,
        output_path: &std::path::Path,
    ) -> Result<PathBuf> {
        let content = self.load_note(filename)?;
        let title = filename.strip_suffix(".md").unwrap_or(filename);

        let mut body_html = String::new();
        for line in content.lines() {
            if let Some(h1) = line.strip_prefix("# ") {
                body_html.push_str(&format!("<h1>{}</h1>\n", h1));
            } else if let Some(h2) = line.strip_prefix("## ") {
                body_html.push_str(&format!("<h2>{}</h2>\n", h2));
            } else if let Some(h3) = line.strip_prefix("### ") {
                body_html.push_str(&format!("<h3>{}</h3>\n", h3));
            } else if line.trim().is_empty() {
                body_html.push_str("<br/>\n");
            } else {
                body_html.push_str(&format!("<p>{}</p>\n", line));
            }
        }

        let html_document = format!(
            "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n<style>body {{ font-family: system-ui, sans-serif; line-height: 1.6; max-width: 800px; margin: 20px auto; padding: 0 20px; }}</style>\n</head>\n<body>\n{}\n</body>\n</html>",
            title, body_html
        );

        let final_path = if output_path.is_dir() {
            output_path.join(format!("{title}.html"))
        } else {
            output_path.to_path_buf()
        };

        std::fs::write(&final_path, html_document)?;
        Ok(final_path)
    }

    /// # Errors
    /// Returns an error if writing zip file fails.
    pub fn export_vault_to_zip(&self, output_zip_path: &std::path::Path) -> Result<usize> {
        let file = std::fs::File::create(output_zip_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        let mut count = 0;
        for filename in self.filenames() {
            if let Ok(content) = self.load_note(&filename) {
                zip.start_file(&filename, options)?;
                use std::io::Write;
                zip.write_all(content.as_bytes())?;
                count += 1;
            }
        }

        zip.finish()?;
        Ok(count)
    }

    /// # Errors
    /// Returns an error if renaming fails.
    pub fn rename_note(&mut self, old_filename: &str, new_title: &str) -> Result<String> {
        let old_path = self.notes_dir.join(old_filename);

        let safe_title = new_title.replace(['/', '\\'], "-");
        let new_filename = format!("{safe_title}.md");
        let new_path = self.notes_dir.join(&new_filename);

        if !old_path.exists() {
            color_eyre::eyre::bail!("Source note does not exist");
        }

        if new_path.exists() {
            color_eyre::eyre::bail!("Destination note already exists");
        }

        std::fs::rename(old_path, new_path)?;

        if let Some(mut meta) = self.metadata.remove(old_filename) {
            meta.modified_at = Utc::now();
            self.metadata.insert(new_filename.clone(), meta);
            self.save_metadata()?;
        }

        Ok(new_filename)
    }

    #[must_use]
    pub fn filenames(&self) -> Vec<String> {
        self.metadata.keys().cloned().collect()
    }

    #[must_use]
    pub fn get_modified_at(&self, filename: &str) -> Option<i64> {
        self.metadata
            .get(filename)
            .map(|m| m.modified_at.timestamp())
    }

    #[must_use]
    pub fn is_pinned(&self, filename: &str) -> bool {
        self.metadata.get(filename).map_or(false, |m| m.is_pinned)
    }

    /// # Errors
    /// Returns an error if saving metadata fails.
    pub fn toggle_pin(&mut self, filename: &str) -> Result<bool> {
        let new_state = if let Some(meta) = self.metadata.get_mut(filename) {
            meta.is_pinned = !meta.is_pinned;
            meta.is_pinned
        } else {
            false
        };
        if self.metadata.contains_key(filename) {
            self.save_metadata()?;
        }
        Ok(new_state)
    }

    /// # Errors
    /// Returns an error if saving metadata fails.
    pub fn set_pinned(&mut self, filename: &str, pinned: bool) -> Result<()> {
        if let Some(meta) = self.metadata.get_mut(filename) {
            meta.is_pinned = pinned;
            self.save_metadata()?;
        }
        Ok(())
    }
}

/// Formats a Unix timestamp into a human-readable local date and time string (`YYYY-MM-DD HH:MM`).
#[must_use]
pub fn format_timestamp(timestamp: i64) -> String {
    if timestamp <= 0 {
        return String::new();
    }
    if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
        let local_dt: chrono::DateTime<chrono::Local> = chrono::DateTime::from(dt);
        local_dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_dir(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("ferronote_test_{name}_{nanos}"));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "");
        assert_eq!(format_timestamp(-100), "");
        let ts = 1753723200; // 2025-07-28 approx
        let formatted = format_timestamp(ts);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("-"));
    }

    #[test]
    fn test_create_and_load_note() {
        let dir = setup_test_dir("create_load");
        let mut store = NoteStore::new(dir).unwrap();
        // Default notes should be auto-created for empty store
        assert_eq!(store.filenames().len(), 2);
        assert!(
            store
                .filenames()
                .contains(&"Welcome to Ferronote.md".to_string())
        );
        assert!(store.filenames().contains(&"Lorem Ipsum.md".to_string()));

        let filename = store.create_note("Test Note").unwrap();
        assert_eq!(filename, "Test Note.md");

        let content = store.load_note(&filename).unwrap();
        assert_eq!(content, "# Test Note\n\n");
    }

    #[test]
    fn test_welcome_note_preserved_on_reopen() {
        let dir = setup_test_dir("welcome_update");
        let mut store1 = NoteStore::new(dir.clone()).unwrap();
        store1
            .save_note("Welcome to Ferronote.md", "User modified content")
            .unwrap();
        assert_eq!(
            store1.load_note("Welcome to Ferronote.md").unwrap(),
            "User modified content"
        );

        // Reopening NoteStore should preserve existing user modifications to Welcome note
        let store2 = NoteStore::new(dir).unwrap();
        let content = store2.load_note("Welcome to Ferronote.md").unwrap();
        assert_eq!(content, "User modified content");
    }

    #[test]
    fn test_default_lorem_ipsum_note_and_backlink() {
        let dir = setup_test_dir("lorem_ipsum_test");
        let store = NoteStore::new(dir).unwrap();
        let welcome_content = store.load_note("Welcome to Ferronote.md").unwrap();
        assert!(welcome_content.contains("[[Lorem Ipsum]]"));

        let lorem_content = store.load_note("Lorem Ipsum.md").unwrap();
        assert!(lorem_content.contains("# Lorem Ipsum"));
        assert!(lorem_content.contains("[[Welcome to Ferronote]]"));

    }

    #[test]
    fn test_save_note() {
        let dir = setup_test_dir("save");
        let mut store = NoteStore::new(dir).unwrap();

        let filename = store.create_note("Save Me").unwrap();
        store.save_note(&filename, "New content!").unwrap();

        let content = store.load_note(&filename).unwrap();
        assert_eq!(content, "New content!");
    }

    #[test]
    fn test_delete_note() {
        let dir = setup_test_dir("delete");
        let mut store = NoteStore::new(dir).unwrap();

        let filename = store.create_note("Delete Me").unwrap();
        assert!(store.metadata.contains_key(&filename));

        store.delete_note(&filename).unwrap();
        assert!(!store.metadata.contains_key(&filename));
        assert!(!store.notes_dir.join(&filename).exists());

        let trash_dir = store.notes_dir.join("trash");
        let trash_files: Vec<_> = std::fs::read_dir(&trash_dir).unwrap().flatten().collect();
        assert_eq!(trash_files.len(), 1);
        assert!(
            trash_files[0]
                .file_name()
                .to_string_lossy()
                .contains("Delete Me.md")
        );
    }

    #[test]
    fn test_rename_note() {
        let dir = setup_test_dir("rename");
        let mut store = NoteStore::new(dir).unwrap();

        let filename = store.create_note("Old Name").unwrap();
        let new_filename = store.rename_note(&filename, "New Name").unwrap();

        assert_eq!(new_filename, "New Name.md");
        assert!(!store.notes_dir.join(&filename).exists());
        assert!(store.notes_dir.join(&new_filename).exists());
        assert!(!store.metadata.contains_key(&filename));
        assert!(store.metadata.contains_key(&new_filename));
    }

    #[test]
    fn test_import_file_and_directory() {
        let store_dir = setup_test_dir("import_store");
        let mut store = NoteStore::new(store_dir).unwrap();

        let source_dir = setup_test_dir("import_source");
        std::fs::create_dir_all(&source_dir).unwrap();
        let file1 = source_dir.join("imported_one.md");
        let file2 = source_dir.join("imported_two.txt");
        std::fs::write(&file1, "# Note One").unwrap();
        std::fs::write(&file2, "# Note Two").unwrap();

        let count = store.import_directory(&source_dir).unwrap();
        assert_eq!(count, 2);
        assert!(store.load_note("imported_one.md").is_ok());
        assert!(store.load_note("imported_two.md").is_ok());
    }

    #[test]
    fn test_restore_and_purge_trash() {
        let dir = setup_test_dir("restore_purge");
        let mut store = NoteStore::new(dir).unwrap();

        let filename = store.create_note("Restore Me").unwrap();
        store.delete_note(&filename).unwrap();

        let trash_list = store.list_trash().unwrap();
        assert_eq!(trash_list.len(), 1);
        let trash_file = &trash_list[0].0;

        let restored = store.restore_note(trash_file).unwrap();
        assert_eq!(restored, "Restore Me.md");
        assert!(store.metadata.contains_key("Restore Me.md"));

        // Delete again and purge
        store.delete_note(&restored).unwrap();
        let purged_count = store.purge_trash().unwrap();
        assert_eq!(purged_count, 1);
        assert!(store.list_trash().unwrap().is_empty());
    }

    #[test]
    fn test_export_html_and_zip() {
        let dir = setup_test_dir("export");
        let mut store = NoteStore::new(dir).unwrap();

        let filename = store.create_note("Export Note").unwrap();
        store
            .save_note(&filename, "# Header 1\n## Header 2\nBody text")
            .unwrap();

        let export_dir = setup_test_dir("export_target");
        let html_target = export_dir.join("export_note.html");
        let exported_html = store.export_note_to_html(&filename, &html_target).unwrap();
        assert!(exported_html.exists());
        let html_content = std::fs::read_to_string(&exported_html).unwrap();
        assert!(html_content.contains("<h1>Header 1</h1>"));
        assert!(html_content.contains("<h2>Header 2</h2>"));

        let zip_target = export_dir.join("vault_export.zip");
        let count = store.export_vault_to_zip(&zip_target).unwrap();
        assert!(count >= 1);
        assert!(zip_target.exists());
    }

    #[test]
    fn test_duplicate_create_title_and_zip_import() {
        let dir = setup_test_dir("dup_and_zip");
        let mut store = NoteStore::new(dir.clone()).unwrap();

        let n1 = store.create_note("Test Title").unwrap();
        assert_eq!(n1, "Test Title.md");

        let err = store.create_note("Test Title");
        assert!(err.is_err());

        // Export vault to zip then import zip into a new store
        let export_dir = setup_test_dir("dup_zip_export");
        let zip_path = export_dir.join("backup.zip");
        store.export_vault_to_zip(&zip_path).unwrap();

        let import_dir = setup_test_dir("dup_zip_import");
        let mut new_store = NoteStore::new(import_dir).unwrap();
        let imported_count = new_store.import_zip(&zip_path).unwrap();
        assert_eq!(imported_count, 3); // Welcome note + Lorem Ipsum + Test Title
        assert!(new_store.filenames().contains(&"Test Title.md".to_string()));
    }

    #[test]
    fn test_pin_toggle_and_persistence() {
        let dir = setup_test_dir("pin_test");
        let mut store = NoteStore::new(dir.clone()).unwrap();
        let filename = store.create_note("Pinned Note").unwrap();

        assert!(!store.is_pinned(&filename));

        // Toggle pin on
        let new_state = store.toggle_pin(&filename).unwrap();
        assert!(new_state);
        assert!(store.is_pinned(&filename));

        // Reopen NoteStore from disk and verify persistence
        let store2 = NoteStore::new(dir).unwrap();
        assert!(store2.is_pinned(&filename));
    }
}
