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
        let ferronote_dir = notes_dir.join(".ferronote");

        if !notes_dir.exists() {
            std::fs::create_dir_all(&notes_dir)?;
        }
        if !ferronote_dir.exists() {
            std::fs::create_dir_all(&ferronote_dir)?;
        }

        let metadata_path = ferronote_dir.join("metadata.json");

        let mut store = Self {
            notes_dir,
            metadata_path,
            metadata: HashMap::new(),
        };

        store.load_metadata()?;
        store.scan_directory()?;

        Ok(store)
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
                && path.extension().is_some_and(|ext| ext == "md")
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                found_files.push(name.to_string());
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
        let path = self.notes_dir.join(filename);

        if path.exists() {
            std::fs::remove_file(path)?;
        }

        self.metadata.remove(filename);
        self.save_metadata()?;

        Ok(())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ferronote_test_{name}"));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        dir
    }

    #[test]
    fn test_create_and_load_note() {
        let dir = setup_test_dir("create_load");
        let mut store = NoteStore::new(dir).unwrap();

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
}
