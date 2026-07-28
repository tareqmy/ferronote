use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Clear, Block, Borders},
    Frame,
};

use crate::{
    action::Action,
    components::{editor::Editor, note_list::NoteList, search_bar::SearchBar},
    event::{Event, EventHandler},
    focus::Focus,
    note_store::NoteStore,
    search::Index,
    tui::Tui,
};
use std::time::Instant;

#[derive(Debug)]
pub struct App<'a> {
    pub should_quit: bool,
    pub note_store: NoteStore,
    pub focus: Focus,
    pub search_bar: SearchBar<'a>,
    pub note_list: NoteList,
    pub editor: Editor<'a>,
    pub index: Index,
    pub last_search_input: Option<Instant>,
    pub current_query: String,
    pub show_help: bool,
}

impl App<'_> {
    #[must_use]
    pub fn new(note_store: NoteStore) -> Self {
        let mut index = Index::new();
        
        for filename in note_store.filenames() {
            if let Ok(content) = note_store.load_note(&filename) {
                let modified_at = note_store.get_modified_at(&filename).unwrap_or(0);
                index.add_note(filename, content, modified_at);
            }
        }

        let mut app = Self {
            should_quit: false,
            note_store,
            focus: Focus::default(),
            search_bar: SearchBar::new(),
            note_list: NoteList::new(),
            editor: Editor::new(),
            index,
            last_search_input: None,
            current_query: String::new(),
            show_help: false,
        };
        app.update_search();
        app
    }

    pub fn update_search(&mut self) {
        let query = self.search_bar.query();
        let mut results = self.index.search(&query);

        if !query.is_empty() {
            let exact_match = results.iter().any(|r| r.title.eq_ignore_ascii_case(&query));
            if !exact_match {
                results.insert(0, crate::search::SearchResult {
                    filename: String::new(),
                    title: format!("Create new note: '{}'", query),
                    score: i64::MAX,
                    title_match_indices: Vec::new(),
                    content_preview: None,
                    is_create_prompt: true,
                    modified_at: chrono::Utc::now().timestamp(),
                });
            }
        }

        self.note_list.set_items(results);
        
        let selected = self.note_list.selected_note();
        self.update(Action::SelectNote(selected));
    }

    pub fn create_note(&mut self, title: &str) -> Result<String> {
        let filename = self.note_store.create_note(title)?;
        let content = format!("# {title}\n\n");
        let modified_at = self.note_store.get_modified_at(&filename).unwrap_or(0);
        self.index.add_note(filename.clone(), content, modified_at);
        self.update_search();
        Ok(filename)
    }

    pub fn save_note(&mut self, filename: &str, content: &str) -> Result<()> {
        self.note_store.save_note(filename, content)?;
        let modified_at = self.note_store.get_modified_at(filename).unwrap_or(0);
        self.index.add_note(filename.to_string(), content.to_string(), modified_at);
        self.update_search();
        Ok(())
    }

    pub fn delete_note(&mut self, filename: &str) -> Result<()> {
        self.note_store.delete_note(filename)?;
        self.index.remove_note(filename);
        self.update_search();
        Ok(())
    }

    pub fn rename_note(&mut self, old_filename: &str, new_title: &str) -> Result<String> {
        let new_filename = self.note_store.rename_note(old_filename, new_title)?;
        if let Ok(content) = self.note_store.load_note(&new_filename) {
            let modified_at = self.note_store.get_modified_at(&new_filename).unwrap_or(0);
            self.index.rename_note(old_filename, new_filename.clone(), content, modified_at);
            self.update_search();
        }
        Ok(new_filename)
    }

    /// # Errors
    /// Returns an error if drawing to the terminal fails or the event stream closes.
    pub async fn run(&mut self, mut tui: Tui, mut events: EventHandler) -> Result<()> {
        while !self.should_quit {
            tui.draw(|frame| self.draw(frame))?;

            let event = events.next().await?;
            let action = self.handle_event(event);

            if let Some(action) = action {
                self.update(action);
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Tick => Some(Action::Tick),
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Mouse(_) => None,
            Event::Resize(w, h) => Some(Action::Resize(w, h)),
            Event::FileChanged(path) => Some(Action::FileChanged(path)),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        if self.show_help {
            return Some(Action::ToggleHelp);
        }

        match key.code {
            KeyCode::Char('q')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                return Some(Action::Quit);
            }
            KeyCode::Char('d')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                return Some(Action::DeleteNote);
            }
            KeyCode::Tab => {
                self.focus = self.focus.next();
                return Some(Action::SaveNote);
            }
            KeyCode::Esc => {
                self.focus = Focus::SearchBar;
                return Some(Action::SaveNote);
            }
            KeyCode::Char('?') if self.focus != Focus::Editor => {
                return Some(Action::ToggleHelp);
            }
            _ => {}
        }

        match self.focus {
            Focus::SearchBar => {
                if key.code == KeyCode::Enter {
                    return Some(Action::SubmitSearch);
                } else if key.code == KeyCode::Down || key.code == KeyCode::Up {
                    // Let search bar down/up jump to note list if needed, or pass it to note_list?
                    // NV behavior: Down arrow in search moves to NoteList.
                    self.focus = Focus::NoteList;
                    if key.code == KeyCode::Down {
                        self.note_list.next();
                    } else {
                        self.note_list.previous();
                    }
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                } else {
                    let old_query = self.search_bar.query();
                    self.search_bar.handle_key(key);
                    let new_query = self.search_bar.query();
                    if old_query != new_query {
                        self.last_search_input = Some(Instant::now());
                    }
                }
            }
            Focus::NoteList => {
                if key.code == KeyCode::Enter {
                    return Some(Action::SubmitSearch);
                } else if key.code == KeyCode::Down {
                    self.note_list.next();
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                } else if key.code == KeyCode::Up {
                    self.note_list.previous();
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
            }
            Focus::Editor => {
                if key.code == KeyCode::Esc {
                    self.focus = Focus::NoteList;
                    return Some(Action::SaveNote);
                }
                self.editor.handle_key(key);
            }
        }

        None
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Action::Quit => {
                self.update(Action::SaveNote);
                self.should_quit = true;
            }
            Action::Tick => {
                if let Some(last_input) = self.last_search_input {
                    if last_input.elapsed().as_millis() >= 50 {
                        self.update_search();
                        self.last_search_input = None;
                    }
                }
                
                if let Some(last_edit) = self.editor.last_edit_time {
                    if last_edit.elapsed().as_secs() >= 1 && self.editor.has_unsaved_changes() {
                        self.update(Action::SaveNote);
                    }
                }
            }
            Action::SelectNote(maybe_filename) => {
                self.update(Action::SaveNote);
                if let Some(filename) = maybe_filename {
                    if filename.is_empty() {
                        // This is a "Create new note" prompt, clear editor content
                        self.editor.set_content("", "");
                    } else if let Ok(content) = self.note_store.load_note(&filename) {
                        let title = filename.strip_suffix(".md").unwrap_or(&filename);
                        self.editor.set_content(title, &content);
                    }
                } else {
                    self.editor.set_content("", "");
                }
            }
            Action::SubmitSearch => {
                self.update(Action::SaveNote);
                if let Some(selected) = self.note_list.selected_note() {
                    if selected.is_empty() {
                        // Create new note flow
                        let query = self.search_bar.query();
                        if !query.is_empty() {
                            if let Ok(filename) = self.create_note(&query) {
                                // Select it explicitly in the list
                                if let Some(idx) = self.note_list.items.iter().position(|r| r.filename == filename) {
                                    self.note_list.state.select(Some(idx));
                                }
                                self.focus = Focus::Editor;
                                self.update(Action::SelectNote(Some(filename)));
                            }
                        }
                    } else {
                        // Jump to existing note
                        self.focus = Focus::Editor;
                        self.update(Action::SelectNote(Some(selected)));
                    }
                }
            }
            Action::SaveNote => {
                if self.editor.has_unsaved_changes() {
                    if let Some(ref note) = self.editor.current_note.clone() {
                        let content = self.editor.content();
                        self.editor.mark_saved();
                        let _ = self.save_note(note, &content);
                    }
                }
            }
            Action::DeleteNote => {
                // Delete currently selected note
                if let Some(selected) = self.note_list.selected_note() {
                    if !selected.is_empty() {
                        let _ = self.delete_note(&selected);
                        
                        // Select the next item in the list automatically
                        let new_selected = self.note_list.selected_note();
                        self.update(Action::SelectNote(new_selected));
                    }
                }
            }
            Action::FileChanged(path) => {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "md" {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            let filename = filename.to_string();
                            if let Ok(content) = self.note_store.load_note(&filename) {
                                let modified_at = self.note_store.get_modified_at(&filename).unwrap_or(0);
                                self.index.add_note(filename.clone(), content.clone(), modified_at);
                                
                                if Some(&filename) == self.editor.current_note.as_ref() {
                                    if !self.editor.has_unsaved_changes() {
                                        let title = filename.strip_suffix(".md").unwrap_or(&filename);
                                        self.editor.set_content(title, &content);
                                    }
                                }
                                self.update_search();
                            }
                        }
                    }
                }
            }
            Action::Render | Action::Resize(_, _) => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(1),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area());

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Note list
                Constraint::Percentage(70), // Editor
            ])
            .split(main_layout[1]);

        // Render components
        self.search_bar
            .draw(frame, main_layout[0], self.focus == Focus::SearchBar);
        self.note_list
            .draw(frame, content_layout[0], self.focus == Focus::NoteList);
        self.editor
            .draw(frame, content_layout[1], self.focus == Focus::Editor);

        // Render dynamic status bar
        let mut spans = vec![
            Span::styled(" [Tab/Esc] ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch Focus  |"),
            Span::styled(" [Ctrl+Q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ];

        if self.focus == Focus::Editor {
            spans.insert(2, Span::styled(" [Ctrl+Z] ", Style::default().fg(Color::Yellow)));
            spans.insert(3, Span::raw("Undo  |"));
            spans.insert(4, Span::styled(" [Ctrl+Y] ", Style::default().fg(Color::Yellow)));
            spans.insert(5, Span::raw("Redo  |"));
            
            let stats = format!("  Words: {} | Chars: {}  |", self.editor.word_count(), self.editor.char_count());
            spans.push(Span::raw(stats));
        } else {
            spans.insert(2, Span::styled(" [?] ", Style::default().fg(Color::Yellow)));
            spans.insert(3, Span::raw("Help  |"));
            spans.insert(4, Span::styled(" [Enter] ", Style::default().fg(Color::Yellow)));
            spans.insert(5, Span::raw("Create/Edit Note  |"));
            spans.insert(6, Span::styled(" [Ctrl+D] ", Style::default().fg(Color::Yellow)));
            spans.insert(7, Span::raw("Delete Note  |"));
        }

        let status_text = Line::from(spans);
        let status_bar = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        frame.render_widget(status_bar, main_layout[2]);

        // Render Help Overlay
        if self.show_help {
            let help_text = vec![
                Line::from(Span::styled("Ferronote Keybindings", Style::default().fg(Color::Yellow))),
                Line::from(""),
                Line::from(vec![Span::styled("Tab / Esc", Style::default().fg(Color::Cyan)), Span::raw(" : Switch Focus")]),
                Line::from(vec![Span::styled("Enter", Style::default().fg(Color::Cyan)), Span::raw("     : Create/Edit Note")]),
                Line::from(vec![Span::styled("Up / Down", Style::default().fg(Color::Cyan)), Span::raw(" : Navigate Note List")]),
                Line::from(vec![Span::styled("Ctrl+Z", Style::default().fg(Color::Cyan)), Span::raw("    : Undo (in Editor)")]),
                Line::from(vec![Span::styled("Ctrl+Y", Style::default().fg(Color::Cyan)), Span::raw("    : Redo (in Editor)")]),
                Line::from(vec![Span::styled("Ctrl+D", Style::default().fg(Color::Cyan)), Span::raw("    : Delete Selected Note")]),
                Line::from(vec![Span::styled("Ctrl+Q", Style::default().fg(Color::Cyan)), Span::raw("    : Quit Application")]),
                Line::from(""),
                Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray))),
            ];

            let help_block = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title(" Help "))
                .alignment(ratatui::layout::Alignment::Center);

            let area = frame.area();
            // Centered rect
            let width = 40;
            let height = 15;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(help_block, popup_area);
        }
    }
}
