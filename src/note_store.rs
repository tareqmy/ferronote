use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteMetadata {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
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

        if store.filenames().is_empty() {
            let _ = store.create_default_welcome_note();
        }

        Ok(store)
    }

    pub fn create_default_welcome_note(&mut self) -> Result<String> {
        let title = "Welcome to Ferronote";
        let filename = self.create_note(title)?;
        let content = r#"# Welcome to Ferronote

Ferronote is a blazing-fast terminal note-taking app inspired by Notational Velocity.

## 🚀 Key Concept: "Search IS Create"
You don't need a separate "New Note" button!
1. Type a note title or search query in the top search bar.
2. If a note with that title exists, it will filter and open it.
3. If no matching note exists, press Enter to instantly create and edit a note with that title.

## ⌨️ Essential Keybindings
- Tab / Esc: Cycle focus between Search Bar, Note List, and Editor.
- Up / Down: Navigate notes in the list.
- PgUp / PgDn: Scroll note list page by page.
- Home / End: Jump to top or bottom of notes list.
- Ctrl+D: Soft delete selected note (moves file to trash).
- Ctrl+Z / Ctrl+Y: Undo / Redo inside the editor.
- Ctrl+Q: Quit Ferronote.
- ?: Toggle Help Overlay.

## 🏷️ Tags & Wiki-Links
- #tags: Add inline tags like #todo or #ideas anywhere in your note, then search #todo to filter instantly.
- [[Wiki-Links]]: Type [[Another Note]] inside a note, place your cursor on it, and press Enter to jump to or create the linked note!

Happy note taking!
"#;
        self.save_note(&filename, content)?;
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

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
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
                if ext == "md" || ext == "txt" {
                    if self.import_file(&path).is_ok() {
                        count += 1;
                    }
                }
            } else if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "trash" {
                    if let Ok(c) = self.import_directory(&path) {
                        count += c;
                    }
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
    fn test_create_and_load_note() {
        let dir = setup_test_dir("create_load");
        let mut store = NoteStore::new(dir).unwrap();
        // Welcome note should be auto-created for empty store
        assert_eq!(store.filenames().len(), 1);
        assert_eq!(store.filenames()[0], "Welcome to Ferronote.md");

        let filename = store.create_note("Test Note").unwrap();
        assert_eq!(filename, "Test Note.md");

        let content = store.load_note(&filename).unwrap();
        assert_eq!(content, "# Test Note\n\n");
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
}
