use ferronote::{action::Action, app::App, config::Config, focus::Focus, note_store::NoteStore};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_end_to_end_note_lifecycle() {
    let temp_vault = tempdir().unwrap();
    let mut store = NoteStore::new(temp_vault.path().to_path_buf()).unwrap();

    // 1. Create initial notes
    let n1 = store.create_note("Rust Architecture").unwrap();
    store
        .save_note(
            &n1,
            "# Rust Architecture\nBuilding CLI app with #rust and #tui.",
        )
        .unwrap();

    let n2 = store.create_note("Weekly Standup").unwrap();
    store
        .save_note(
            &n2,
            "# Weekly Standup\nReviewed [[Rust Architecture]] progress. #work #urgent",
        )
        .unwrap();

    // 2. Initialize App
    let mut app = App::new(store);
    assert_eq!(app.focus, Focus::SearchBar);

    // 3. Search as-you-type and Tag Filtering
    app.search_bar.textarea.select_all();
    app.search_bar.textarea.insert_str("#rust");
    app.update_search();

    let search_results = app.note_list.items.clone();
    assert_eq!(search_results.len(), 2); // 1 create prompt + 1 matching note
    assert!(search_results[0].is_create_prompt);
    assert_eq!(search_results[1].filename, "Rust Architecture.md");

    // 4. Backlinks tracking
    let backlinks = app.index.get_backlinks("Rust Architecture");
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].filename, "Weekly Standup.md");

    // 5. Search IS Create flow
    app.search_bar.textarea.select_all();
    app.search_bar.textarea.insert_str("Brand New Note");
    app.update_search();

    assert!(app.note_list.items[0].is_create_prompt);
    app.update(Action::SubmitSearch);

    assert_eq!(app.focus, Focus::Editor);
    assert_eq!(
        app.editor.current_note,
        Some("Brand New Note.md".to_string())
    );
    assert!(
        app.note_store
            .filenames()
            .contains(&"Brand New Note.md".to_string())
    );

    // 6. Soft Delete and Trash Management
    let initial_count = app.note_store.filenames().len();
    app.update(Action::DeleteNote);

    assert_eq!(app.note_store.filenames().len(), initial_count - 1);
    let trash_items = app.note_store.list_trash().unwrap();
    assert_eq!(trash_items.len(), 1);
    let (trash_filename, original_title) = &trash_items[0];
    assert_eq!(original_title, "Brand New Note");

    // Restore from trash
    let restored = app.note_store.restore_note(trash_filename).unwrap();
    assert_eq!(restored, "Brand New Note.md");
    assert_eq!(app.note_store.filenames().len(), initial_count);

    // Soft delete again and Purge trash
    let (to_delete, _) = app
        .note_store
        .list_trash()
        .unwrap()
        .first()
        .cloned()
        .unwrap_or_default();
    if to_delete.is_empty() {
        app.note_store.delete_note("Brand New Note.md").unwrap();
    }
    let purged = app.note_store.purge_trash().unwrap();
    assert!(purged >= 1);
    assert!(app.note_store.list_trash().unwrap().is_empty());
}

#[test]
fn test_export_and_import_vault_workflow() {
    let source_vault = tempdir().unwrap();
    let mut store = NoteStore::new(source_vault.path().to_path_buf()).unwrap();

    let note1 = store.create_note("Document One").unwrap();
    store
        .save_note(&note1, "# Document One\nContent for export test.")
        .unwrap();

    let export_dir = tempdir().unwrap();

    // Export single note to HTML
    let html_path = export_dir.path().join("doc1.html");
    let exported_html = store.export_note_to_html(&note1, &html_path).unwrap();
    assert!(exported_html.exists());
    let html_content = fs::read_to_string(&exported_html).unwrap();
    assert!(html_content.contains("<h1>Document One</h1>"));

    // Export entire vault to ZIP
    let zip_path = export_dir.path().join("vault.zip");
    let count = store.export_vault_to_zip(&zip_path).unwrap();
    assert!(count >= 3); // Welcome note + Lorem Ipsum + Document One

    // Import ZIP into fresh vault
    let target_vault = tempdir().unwrap();
    let mut new_store = NoteStore::new(target_vault.path().to_path_buf()).unwrap();
    let imported_count = new_store.import_zip(&zip_path).unwrap();
    assert_eq!(imported_count, count);
    assert!(
        new_store
            .filenames()
            .contains(&"Document One.md".to_string())
    );
}

#[test]
fn test_settings_overlay_and_config_persistence() {
    let vault_dir = tempdir().unwrap();
    let store = NoteStore::new(vault_dir.path().to_path_buf()).unwrap();
    let mut app = App::new(store);

    app.config.notes_dir = vault_dir.path().to_path_buf();

    // Toggle settings overlay
    app.update(Action::ToggleSettings);
    assert!(app.show_settings);

    // Modify default extension (Option 0)
    let old_ext = app.config.default_extension.clone();
    app.update(Action::ChangeSettingOption(true));
    assert_ne!(app.config.default_extension, old_ext);

    // Navigate to Theme (Option 4)
    for _ in 0..4 {
        app.update(Action::NextSetting);
    }
    assert_eq!(app.settings_selected_index, 4);

    let old_theme = app.config.theme.clone();
    app.update(Action::ChangeSettingOption(true));
    assert_ne!(app.config.theme, old_theme);

    // Close settings overlay
    app.update(Action::ToggleSettings);
    assert!(!app.show_settings);

    // Verify config file was persisted to disk
    let config_path = vault_dir.path().join("config.json");
    assert!(config_path.exists());
    let saved_json = fs::read_to_string(config_path).unwrap();
    let loaded_config: Config = serde_json::from_str(&saved_json).unwrap();
    assert_eq!(loaded_config.theme, app.config.theme);
}
