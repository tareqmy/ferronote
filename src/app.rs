use color_eyre::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    action::Action,
    components::{editor::Editor, note_list::NoteList, search_bar::SearchBar},
    config::Config,
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
    pub show_settings: bool,
    pub settings_selected_index: usize,
    pub config: Config,
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

        let config = Config::load().unwrap_or_default();

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
            show_settings: false,
            settings_selected_index: 0,
            config,
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
                results.insert(
                    0,
                    crate::search::SearchResult {
                        filename: String::new(),
                        title: format!("Create new note: '{}'", query),
                        score: i64::MAX,
                        title_match_indices: Vec::new(),
                        content_preview: None,
                        is_create_prompt: true,
                        modified_at: chrono::Utc::now().timestamp(),
                    },
                );
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
        self.index
            .add_note(filename.to_string(), content.to_string(), modified_at);
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
            self.index
                .rename_note(old_filename, new_filename.clone(), content, modified_at);
            self.update_search();
        }
        Ok(new_filename)
    }

    fn cycle_setting(&mut self, forward: bool) {
        match self.settings_selected_index {
            0 => {
                self.config.default_extension = if self.config.default_extension == "md" {
                    "txt".to_string()
                } else {
                    "md".to_string()
                };
            }
            1 => {
                let options = [500, 1000, 2000, 3000];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.auto_save_delay_ms)
                    .unwrap_or(1);
                let next_idx = if forward {
                    (current_idx + 1) % options.len()
                } else {
                    (current_idx + options.len() - 1) % options.len()
                };
                self.config.auto_save_delay_ms = options[next_idx];
            }
            2 => {
                let options = [2, 4, 8];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.tab_size)
                    .unwrap_or(1);
                let next_idx = if forward {
                    (current_idx + 1) % options.len()
                } else {
                    (current_idx + options.len() - 1) % options.len()
                };
                self.config.tab_size = options[next_idx];
            }
            3 => {
                let options = [20, 25, 30, 35, 40];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.sidebar_width_percent)
                    .unwrap_or(2);
                let next_idx = if forward {
                    (current_idx + 1) % options.len()
                } else {
                    (current_idx + options.len() - 1) % options.len()
                };
                self.config.sidebar_width_percent = options[next_idx];
            }
            4 => {
                let options = ["default", "gruvbox", "nord", "dracula"];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.theme)
                    .unwrap_or(0);
                let next_idx = if forward {
                    (current_idx + 1) % options.len()
                } else {
                    (current_idx + options.len() - 1) % options.len()
                };
                self.config.theme = options[next_idx].to_string();
            }
            5 => {
                let options = ["modified_desc", "title_asc", "created_desc"];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.default_sort)
                    .unwrap_or(0);
                let next_idx = if forward {
                    (current_idx + 1) % options.len()
                } else {
                    (current_idx + options.len() - 1) % options.len()
                };
                self.config.default_sort = options[next_idx].to_string();
            }
            6 => {
                let options = [0, 7, 14, 30, 90];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.auto_purge_days)
                    .unwrap_or(3);
                let next_idx = if forward {
                    (current_idx + 1) % options.len()
                } else {
                    (current_idx + options.len() - 1) % options.len()
                };
                self.config.auto_purge_days = options[next_idx];
            }
            _ => {}
        }
        let _ = self.config.save();
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
        if key.kind != crossterm::event::KeyEventKind::Press {
            return None;
        }

        if self.show_settings {
            match key.code {
                KeyCode::Esc | KeyCode::F(2) => return Some(Action::ToggleSettings),
                KeyCode::Char('p')
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    return Some(Action::ToggleSettings);
                }
                KeyCode::Up | KeyCode::Char('k') => return Some(Action::PrevSetting),
                KeyCode::Down | KeyCode::Char('j') => return Some(Action::NextSetting),
                KeyCode::Left | KeyCode::Char('h') => {
                    return Some(Action::ChangeSettingOption(false));
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                    return Some(Action::ChangeSettingOption(true));
                }
                _ => return None,
            }
        }

        if self.show_help {
            return Some(Action::ToggleHelp);
        }

        if (key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
            && key.code == KeyCode::Char('p'))
            || key.code == KeyCode::F(2)
        {
            return Some(Action::ToggleSettings);
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
                } else if matches!(
                    key.code,
                    KeyCode::Down | KeyCode::Up | KeyCode::PageDown | KeyCode::PageUp
                ) {
                    self.focus = Focus::NoteList;
                    match key.code {
                        KeyCode::Down => self.note_list.next(),
                        KeyCode::Up => self.note_list.previous(),
                        KeyCode::PageDown => self.note_list.page_down(10),
                        KeyCode::PageUp => self.note_list.page_up(10),
                        _ => {}
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
            Focus::NoteList => match key.code {
                KeyCode::Enter => return Some(Action::SubmitSearch),
                KeyCode::Down => {
                    self.note_list.next();
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
                KeyCode::Up => {
                    self.note_list.previous();
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
                KeyCode::PageDown => {
                    self.note_list.page_down(10);
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
                KeyCode::PageUp => {
                    self.note_list.page_up(10);
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
                KeyCode::Home => {
                    self.note_list.select_first();
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
                KeyCode::End => {
                    self.note_list.select_last();
                    return Some(Action::SelectNote(self.note_list.selected_note()));
                }
                _ => {}
            },
            Focus::Editor => {
                if key.code == KeyCode::Esc {
                    self.focus = Focus::NoteList;
                    return Some(Action::SaveNote);
                } else if key.code == KeyCode::Enter {
                    if let Some(link) = self.editor.extract_wiki_link_at_cursor() {
                        self.update(Action::SaveNote);

                        let filename = format!("{}.md", link.replace(['/', '\\'], "-"));
                        if !self.note_store.filenames().contains(&filename) {
                            let _ = self.create_note(&link);
                        }

                        let mut ta = tui_textarea::TextArea::default();
                        ta.insert_str(&link);
                        self.search_bar.textarea = ta;
                        self.update_search();

                        return None;
                    }
                }
                self.editor.handle_key(key);
            }
        }

        None
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::ToggleSettings => {
                self.show_settings = !self.show_settings;
            }
            Action::NextSetting => {
                if self.settings_selected_index < 6 {
                    self.settings_selected_index += 1;
                }
            }
            Action::PrevSetting => {
                self.settings_selected_index = self.settings_selected_index.saturating_sub(1);
            }
            Action::ChangeSettingOption(forward) => {
                self.cycle_setting(forward);
            }
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
                        self.editor.set_content(&filename, &content);
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
                                if let Some(idx) = self
                                    .note_list
                                    .items
                                    .iter()
                                    .position(|r| r.filename == filename)
                                {
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
                                let modified_at =
                                    self.note_store.get_modified_at(&filename).unwrap_or(0);
                                self.index
                                    .add_note(filename.clone(), content.clone(), modified_at);

                                if Some(&filename) == self.editor.current_note.as_ref() {
                                    if !self.editor.has_unsaved_changes() {
                                        self.editor.set_content(&filename, &content);
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
        let theme = crate::theme::ThemePalette::from_name(&self.config.theme);

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(1),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area());

        let sidebar_width = self.config.sidebar_width_percent.clamp(10, 80);
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(sidebar_width),
                Constraint::Percentage(100 - sidebar_width),
            ])
            .split(main_layout[1]);

        // Render components
        self.search_bar.draw(
            frame,
            main_layout[0],
            self.focus == Focus::SearchBar,
            &theme,
        );
        self.note_list.draw(
            frame,
            content_layout[0],
            self.focus == Focus::NoteList,
            &theme,
        );
        self.editor.draw(
            frame,
            content_layout[1],
            self.focus == Focus::Editor,
            &theme,
        );

        // Render dynamic status bar (no background color, uses terminal default)
        let key_style = Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD);
        let text_style = Style::default().fg(Color::Reset);
        let sep_style = Style::default().fg(theme.border_inactive);

        let mut spans = vec![
            Span::styled(" [Tab/Esc] ", key_style),
            Span::styled("Switch Focus", text_style),
            Span::styled(" │", sep_style),
        ];

        if self.focus == Focus::Editor {
            spans.push(Span::styled(" [Ctrl+Z] ", key_style));
            spans.push(Span::styled("Undo", text_style));
            spans.push(Span::styled(" │", sep_style));

            spans.push(Span::styled(" [Ctrl+Y] ", key_style));
            spans.push(Span::styled("Redo", text_style));
            spans.push(Span::styled(" │", sep_style));

            spans.push(Span::styled(" [Ctrl+Q] ", key_style));
            spans.push(Span::styled("Quit", text_style));
            spans.push(Span::styled(" │", sep_style));

            let backlinks_count = if let Some(ref note) = self.editor.current_note {
                self.index.get_backlinks(note).len()
            } else {
                0
            };

            let stats = format!(
                " Words: {} │ Chars: {} │ Backlinks: {} ",
                self.editor.word_count(),
                self.editor.char_count(),
                backlinks_count
            );
            spans.push(Span::styled(stats, Style::default().fg(Color::Yellow)));
        } else {
            spans.push(Span::styled(" [?] ", key_style));
            spans.push(Span::styled("Help", text_style));
            spans.push(Span::styled(" │", sep_style));

            spans.push(Span::styled(" [Enter] ", key_style));
            spans.push(Span::styled("Create/Edit", text_style));
            spans.push(Span::styled(" │", sep_style));

            spans.push(Span::styled(" [Ctrl+D] ", key_style));
            spans.push(Span::styled("Delete", text_style));
            spans.push(Span::styled(" │", sep_style));

            spans.push(Span::styled(" [Ctrl+Q] ", key_style));
            spans.push(Span::styled("Quit", text_style));
        }

        let status_text = Line::from(spans);
        let status_bar = Paragraph::new(status_text);
        frame.render_widget(status_bar, main_layout[2]);

        // Render Help Overlay
        if self.show_help {
            let help_text = vec![
                Line::from(Span::styled(
                    "Ferronote Keybindings",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Tab / Esc", Style::default().fg(Color::Cyan)),
                    Span::raw(" : Switch Focus"),
                ]),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                    Span::raw("     : Create/Edit Note"),
                ]),
                Line::from(vec![
                    Span::styled("Up / Down", Style::default().fg(Color::Cyan)),
                    Span::raw(" : Navigate Note List"),
                ]),
                Line::from(vec![
                    Span::styled("PgUp / PgDn", Style::default().fg(Color::Cyan)),
                    Span::raw(" : Scroll Note List Page"),
                ]),
                Line::from(vec![
                    Span::styled("Home / End", Style::default().fg(Color::Cyan)),
                    Span::raw("  : Jump to Top / Bottom"),
                ]),
                Line::from(vec![
                    Span::styled("Ctrl+Z", Style::default().fg(Color::Cyan)),
                    Span::raw("    : Undo (in Editor)"),
                ]),
                Line::from(vec![
                    Span::styled("Ctrl+Y", Style::default().fg(Color::Cyan)),
                    Span::raw("    : Redo (in Editor)"),
                ]),
                Line::from(vec![
                    Span::styled("Ctrl+D", Style::default().fg(Color::Cyan)),
                    Span::raw("    : Delete Selected Note"),
                ]),
                Line::from(vec![
                    Span::styled("F2 / Ctrl+P", Style::default().fg(Color::Cyan)),
                    Span::raw(" : Settings Overlay"),
                ]),
                Line::from(vec![
                    Span::styled("Ctrl+Q", Style::default().fg(Color::Cyan)),
                    Span::raw("    : Quit Application"),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Press any key to close",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let help_block = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title(" Help "))
                .alignment(ratatui::layout::Alignment::Center);

            let area = frame.area();
            let width = 42;
            let height = 16;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(help_block, popup_area);
        }

        // Render Settings Overlay (Interactive & Left-Aligned)
        if self.show_settings {
            let options_data = [
                (
                    "Default Extension",
                    format!(".{}", self.config.default_extension),
                ),
                (
                    "Auto-Save Delay",
                    format!("{} ms", self.config.auto_save_delay_ms),
                ),
                ("Tab Size", format!("{} spaces", self.config.tab_size)),
                (
                    "Sidebar Width",
                    format!("{}%", self.config.sidebar_width_percent),
                ),
                ("Active Theme", self.config.theme.clone()),
                ("Default Sort", self.config.default_sort.clone()),
                (
                    "Auto-Purge Trash",
                    if self.config.auto_purge_days == 0 {
                        "Disabled".to_string()
                    } else {
                        format!("{} days", self.config.auto_purge_days)
                    },
                ),
            ];

            let mut lines = vec![
                Line::from(Span::styled(
                    " ⚙️  Ferronote Settings",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (idx, (label, val)) in options_data.iter().enumerate() {
                let is_selected = idx == self.settings_selected_index;
                let prefix = if is_selected { " ▶ " } else { "   " };
                let label_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                };

                let val_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                lines.push(Line::from(vec![
                    Span::styled(prefix, label_style),
                    Span::styled(format!("{:<20}", label), label_style),
                    Span::styled(format!(" [ {:<14} ]", val), val_style),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " [Up/Down] Select   [Left/Right/Enter] Modify   [Esc/F2] Close",
                Style::default().fg(Color::DarkGray),
            )));

            let settings_block = Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Settings (F2) "),
            );

            let area = frame.area();
            let width = 64;
            let height = 14;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(settings_block, popup_area);
        }
    }
}
