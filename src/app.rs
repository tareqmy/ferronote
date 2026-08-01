use color_eyre::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph,
    },
};

use crate::{
    action::Action,
    components::{editor::Editor, note_list::NoteList, search_bar::SearchBar},
    config::Config,
    event::{Event, EventHandler},
    focus::Focus,
    tui::Tui,
};
use ferronote_store::{NoteStore, format_timestamp};
use ferronote_search::Index;
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
    pub show_about: bool,
    pub show_settings: bool,
    pub show_delete_confirmation: bool,
    pub show_rename_prompt: bool,
    pub popup_stack: crate::components::popup_stack::PopupStack,
    pub rename_input: tui_textarea::TextArea<'a>,
    pub show_more_shortcuts: bool,
    pub show_notes_list: bool,
    pub settings_selected_index: usize,
    pub config: Config,
    pub env: crate::environment::Environment,
    pub search_area: Rect,
    pub content_area: Rect,
    pub list_area: Rect,
    pub editor_area: Rect,
    pub latest_version: Option<String>,
    pub update_area: Option<Rect>,
    pub is_resizing_sidebar: bool,
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
        let env = crate::environment::Environment::new();

        let mut app = Self {
            should_quit: false,
            note_store,
            focus: Focus::default(),
            search_bar: SearchBar::new(env.queue.clone()),
            note_list: NoteList::new(env.queue.clone()),
            editor: Editor::new(env.queue.clone()),
            index,
            last_search_input: None,
            current_query: String::new(),
            show_about: false,
            show_settings: false,
            show_delete_confirmation: false,
            show_rename_prompt: false,
            popup_stack: crate::components::popup_stack::PopupStack::new(env.queue.clone()),
            rename_input: tui_textarea::TextArea::default(),
            show_more_shortcuts: false,
            show_notes_list: true,
            settings_selected_index: 0,
            config,
            env,
            search_area: Rect::default(),
            content_area: Rect::default(),
            list_area: Rect::default(),
            editor_area: Rect::default(),
            latest_version: None,
            update_area: None,
            is_resizing_sidebar: false,
        };
        app.update_search();
        app
    }

    pub fn update_search(&mut self) {
        let query = self.search_bar.query();
        let mut results = self.index.search(&query, &self.config.default_sort);

        if !query.is_empty() {
            results.insert(
                0,
                ferronote_search::SearchResult {
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

        self.note_list.set_items(results);

        if let Some(ref current) = self.editor.current_note
            && let Some(idx) = self.note_list.items.iter().position(|r| r.filename == *current)
        {
            self.note_list.state.select(Some(idx));
        }

        let selected = self.note_list.selected_note();
        self.update(Action::SelectNote(selected));
    }

    pub fn create_note(&mut self, title: &str) -> Result<String> {
        let filename = match self.note_store.create_note(title) {
            Ok(f) => f,
            Err(_) => {
                let safe_title = title.replace(['/', '\\'], "-");
                let mut count = 1;
                loop {
                    let candidate = format!("{safe_title} {count}");
                    if let Ok(f) = self.note_store.create_note(&candidate) {
                        break f;
                    }
                    count += 1;
                }
            }
        };
        let content = format!("# {title}\n\n");
        let modified_at = self.note_store.get_modified_at(&filename).unwrap_or(0);
        self.index.add_note(filename.clone(), content.clone(), modified_at);
        self.editor.set_content(&filename, &content);
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
                let options = [20, 25, 30, 38, 40];
                let current_idx = options
                    .iter()
                    .position(|&v| v == self.config.sidebar_width_percent)
                    .unwrap_or(3);
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
                let options = ["modified_desc", "modified_asc", "title_asc", "title_desc", "created_desc", "created_asc"];
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
                self.update_search();
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
            7 => {
                self.config.show_modified_time = !self.config.show_modified_time;
            }
            8 => {
                self.config.word_wrap = !self.config.word_wrap;
            }
            9 => {
                self.config.notes_list_position = if self.config.notes_list_position == "top" {
                    "left".to_string()
                } else {
                    "top".to_string()
                };
            }
            10 => {
                self.config.mouse_capture = !self.config.mouse_capture;
                if self.config.mouse_capture {
                    let _ = crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture);
                } else {
                    let _ = crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture);
                }
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
                self.env.queue.push(action);
            }

            while let Some(action) = self.env.queue.pop() {
                if action == Action::OpenExternalEditor {
                    self.update(Action::SaveNote);
                    
                    if let Some(current) = self.editor.current_note.clone() {
                        // Suspend TUI
                        crate::tui::Tui::restore()?;
                        
                        // Determine editor
                        let editor_cmd = if self.config.external_editor == "auto" {
                            std::env::var("VISUAL")
                                .or_else(|_| std::env::var("EDITOR"))
                                .unwrap_or_else(|_| {
                                    if cfg!(windows) {
                                        "notepad".to_string()
                                    } else {
                                        "vi".to_string()
                                    }
                                })
                        } else {
                            self.config.external_editor.clone()
                        };
                            
                        let path = self.note_store.get_note_path(&current);
                        
                        // Spawn editor
                        let _ = std::process::Command::new(editor_cmd)
                            .arg(path)
                            .spawn()
                            .and_then(|mut child| child.wait());
                            
                        // Restore TUI
                        tui = crate::tui::Tui::init(self.config.mouse_capture)?;
                        
                        // Reload content
                        if let Ok(content) = self.note_store.load_note(&current) {
                            self.editor.set_content(&current, &content);
                        }
                    }
                } else {
                    self.update(action);
                }
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        if !self.popup_stack.is_empty() {
            use crate::components::Component;
            match event {
                Event::Key(_) | Event::Mouse(_) => {
                    if let Ok(crate::components::EventState::Consumed) = self.popup_stack.event(&event) {
                        return None;
                    }
                }
                _ => {}
            }
        }

        match event {
            Event::Tick => Some(Action::Tick),
            Event::Key(key_event) => self.handle_key(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse(mouse_event),
            Event::Resize(w, h) => Some(Action::Resize(w, h)),
            Event::FileChanged(path) => Some(Action::FileChanged(path)),
            Event::NewVersion(version) => Some(Action::NewVersion(version)),
        }
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> Option<Action> {
        match mouse.kind {
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                Some(Action::MouseDown(mouse.column, mouse.row))
            }
            crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                Some(Action::MouseDrag(mouse.column, mouse.row))
            }
            crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                Some(Action::MouseUp(mouse.column, mouse.row))
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                Some(Action::MouseScrollUp(mouse.column, mouse.row))
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                Some(Action::MouseScrollDown(mouse.column, mouse.row))
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn is_on_divider(&self, x: u16, y: u16) -> bool {
        if !self.show_notes_list {
            return false;
        }

        if self.config.notes_list_position == "left" {
            let split_x = self.list_area.x + self.list_area.width;
            let on_x = x >= split_x.saturating_sub(1) && x <= split_x.saturating_add(1);
            let on_y = y >= self.content_area.y
                && y < self.content_area.y + self.content_area.height;
            on_x && on_y
        } else {
            let split_y = self.list_area.y + self.list_area.height;
            let on_y = y >= split_y.saturating_sub(1) && y <= split_y.saturating_add(1);
            let on_x = x >= self.content_area.x
                && x < self.content_area.x + self.content_area.width;
            on_x && on_y
        }
    }

    fn update_sidebar_width_percent(&mut self, x: u16, y: u16) {
        if self.config.notes_list_position == "left" {
            let rel_x = x.saturating_sub(self.content_area.x);
            if self.content_area.width > 0 {
                let pct = ((rel_x as u32 * 100) / self.content_area.width as u32) as u16;
                self.config.sidebar_width_percent = pct.clamp(20, 40);
            }
        } else {
            let rel_y = y.saturating_sub(self.content_area.y);
            if self.content_area.height > 0 {
                let pct = ((rel_y as u32 * 100) / self.content_area.height as u32) as u16;
                self.config.sidebar_width_percent = pct.clamp(20, 40);
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        if key.kind != crossterm::event::KeyEventKind::Press {
            return None;
        }

        if self.show_settings {
            match key.code {
                KeyCode::Esc => return Some(Action::ToggleSettings),
                KeyCode::Char('p')
                    if key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    return Some(Action::ToggleSettings);
                }
                _ => {}
            }

            if self.settings_selected_index == 11 {
                match key.code {
                    KeyCode::Up => return Some(Action::PrevSetting),
                    KeyCode::Down => return Some(Action::NextSetting),
                    KeyCode::PageUp => return Some(Action::PageUpSetting),
                    KeyCode::PageDown => return Some(Action::PageDownSetting),
                    KeyCode::Backspace => return Some(Action::SettingsBackspace),
                    KeyCode::Char(c) => return Some(Action::SettingsTypeChar(c)),
                    _ => return None,
                }
            } else {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => return Some(Action::PrevSetting),
                    KeyCode::Down | KeyCode::Char('j') => return Some(Action::NextSetting),
                    KeyCode::PageUp => return Some(Action::PageUpSetting),
                    KeyCode::PageDown => return Some(Action::PageDownSetting),
                    KeyCode::Home => return Some(Action::FirstSetting),
                    KeyCode::End => return Some(Action::LastSetting),
                    KeyCode::Left | KeyCode::Char('h') => {
                        return Some(Action::ChangeSettingOption(false));
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                        return Some(Action::ChangeSettingOption(true));
                    }
                    _ => return None,
                }
            }
        }

        if self.show_rename_prompt {
            match key.code {
                KeyCode::Esc => return Some(Action::CancelRenameNote),
                KeyCode::Enter => {
                    let text = self.rename_input.lines().join("");
                    return Some(Action::SubmitRenameNote(text));
                }
                _ => {
                    self.rename_input.input(key);
                    return None;
                }
            }
        }

        if self.show_delete_confirmation {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    return Some(Action::DeleteNote);
                }
                _ => return Some(Action::CancelDeleteNote),
            }
        }



        if self.show_about {
            return Some(Action::ToggleAbout);
        }

        if key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            match key.code {
                KeyCode::Char('v') => return Some(Action::ToggleAbout),
                KeyCode::Char('p') => return Some(Action::ToggleSettings),
                KeyCode::Char('q') => return Some(Action::Quit),
                KeyCode::Char('r') => return Some(Action::PromptRenameNote),
                KeyCode::Char('d') => return Some(Action::PromptDeleteNote),
                KeyCode::Char('b') => return Some(Action::ToggleNotesList),
                KeyCode::Char('e') => return Some(Action::ToggleMoreShortcuts),
                KeyCode::Char('s') => return Some(Action::SaveNote),
                KeyCode::Char('o') => return Some(Action::OpenExternalEditor),
                KeyCode::Char('n') => {
                    self.search_bar.clear();
                    self.update_search();
                    self.focus = Focus::SearchBar;
                    return Some(Action::SaveNote);
                }
                KeyCode::Char('l') => {
                    self.focus = Focus::SearchBar;
                    return None;
                }
                _ => {}
            }
        }

        match key.code {

            KeyCode::Tab => {
                self.focus = self.focus.next();
                return Some(Action::SaveNote);
            }
            KeyCode::BackTab => {
                self.focus = self.focus.previous();
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
                    use crate::components::Component;
                    let old_query = self.search_bar.query();
                    let _ = self.search_bar.event(&crate::event::Event::Key(key));
                    let new_query = self.search_bar.query();
                    if old_query != new_query {
                        self.last_search_input = Some(Instant::now());
                    }
                }
            }
            Focus::NoteList => {
                use crate::components::Component;
                if key.code == KeyCode::Enter {
                    return Some(Action::SubmitSearch);
                }
                let _ = self.note_list.event(&crate::event::Event::Key(key));
            }
            Focus::Editor => {
                if key.code == KeyCode::Esc {
                    if self.editor.is_editing {
                        self.editor.is_editing = false;
                        return None;
                    }
                    if self.editor.search_textarea.is_some() {
                        self.editor.search_textarea = None;
                        return None;
                    }
                    use crate::components::Component;
                    let _ = self.editor.event(&crate::event::Event::Key(key));
                    return None;
                } else if key.code == KeyCode::Enter
                    && let Some(link) = self.editor.extract_wiki_link_at_cursor()
                {
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
                use crate::components::Component;
                let _ = self.editor.event(&crate::event::Event::Key(key));
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
                if self.settings_selected_index < 11 {
                    self.settings_selected_index += 1;
                }
            }
            Action::PrevSetting => {
                self.settings_selected_index = self.settings_selected_index.saturating_sub(1);
            }
            Action::PageUpSetting => {
                self.settings_selected_index = self.settings_selected_index.saturating_sub(5);
            }
            Action::PageDownSetting => {
                self.settings_selected_index = std::cmp::min(self.settings_selected_index.saturating_add(5), 11);
            }
            Action::FirstSetting => {
                self.settings_selected_index = 0;
            }
            Action::LastSetting => {
                self.settings_selected_index = 11;
            }
            Action::ChangeSettingOption(forward) => {
                self.cycle_setting(forward);
            }
            Action::SettingsTypeChar(c) => {
                if self.settings_selected_index == 11 {
                    self.config.external_editor.push(c);
                    let _ = self.config.save();
                }
            }
            Action::SettingsBackspace => {
                if self.settings_selected_index == 11 {
                    self.config.external_editor.pop();
                    let _ = self.config.save();
                }
            }
            Action::ToggleHelp => {
                if !self.popup_stack.is_empty() {
                    self.popup_stack.clear();
                } else {
                    self.popup_stack.push(crate::components::popup_stack::AnyPopup::Help(crate::components::help::HelpPopup::new(self.env.queue.clone())));
                }
            }
            Action::ToggleAbout => {
                self.show_about = !self.show_about;
            }
            Action::ToggleMoreShortcuts => {
                self.show_more_shortcuts = !self.show_more_shortcuts;
            }
            Action::ToggleNotesList => {
                self.show_notes_list = !self.show_notes_list;
                if !self.show_notes_list && self.focus == Focus::NoteList {
                    self.focus = Focus::SearchBar;
                }
            }
            Action::Quit => {
                self.update(Action::SaveNote);
                self.should_quit = true;
            }
            Action::NewVersion(version) => {
                self.latest_version = Some(version);
            }
            Action::UpdateApp => {
                self.trigger_update();
                self.should_quit = true;
            }
            Action::Tick => {
                if let Some(last_input) = self.last_search_input
                    && last_input.elapsed().as_millis() >= 50
                {
                    self.update_search();
                    self.last_search_input = None;
                }

                if let Some(last_edit) = self.editor.last_edit_time
                    && last_edit.elapsed().as_millis() >= u128::from(self.config.auto_save_delay_ms)
                    && self.editor.has_unsaved_changes()
                {
                    self.update(Action::SaveNote);
                }
            }
            Action::ToggleEditMode => {
                self.editor.toggle_edit_mode();
            }
            Action::OpenExternalEditor => {
                // Handled in `App::run`
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
                        if !query.is_empty()
                            && let Ok(filename) = self.create_note(&query)
                        {
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
                    } else {
                        // Jump to existing note
                        self.focus = Focus::Editor;
                        self.update(Action::SelectNote(Some(selected)));
                    }
                }
            }
            Action::SaveNote => {
                if self.editor.has_unsaved_changes()
                    && let Some(ref note) = self.editor.current_note.clone()
                {
                    let content = self.editor.content();
                    self.editor.mark_saved();
                    let _ = self.save_note(note, &content);
                }
            }
            Action::PromptDeleteNote => {
                if let Some(selected) = self.note_list.selected_note() {
                    if !selected.is_empty() {
                        self.show_delete_confirmation = true;
                    }
                }
            }
            Action::CancelDeleteNote => {
                self.show_delete_confirmation = false;
            }
            Action::DeleteNote => {
                self.show_delete_confirmation = false;
                // Delete currently selected note
                if let Some(selected) = self.note_list.selected_note()
                    && !selected.is_empty()
                {
                    let _ = self.delete_note(&selected);

                    // Select the next item in the list automatically
                    let new_selected = self.note_list.selected_note();
                    self.update(Action::SelectNote(new_selected));
                }
            }
            Action::PromptRenameNote => {
                if let Some(selected) = self.note_list.selected_note() {
                    if !selected.is_empty() {
                        self.show_rename_prompt = true;
                        let title = self
                            .note_list
                            .items
                            .iter()
                            .find(|i| i.filename == selected)
                            .map(|i| i.title.clone())
                            .unwrap_or_else(|| {
                                selected.strip_suffix(".md").unwrap_or(&selected).to_string()
                            });
                        let mut textarea = tui_textarea::TextArea::default();
                        textarea.insert_str(&title);
                        self.rename_input = textarea;
                    }
                }
            }
            Action::CancelRenameNote => {
                self.show_rename_prompt = false;
            }
            Action::SubmitRenameNote(new_title) => {
                self.show_rename_prompt = false;
                if let Some(old_filename) = self.note_list.selected_note() {
                    if !new_title.is_empty() {
                        let _ = self.rename_note(&old_filename, &new_title);
                        self.update_search();
                    }
                }
            }
            Action::FileChanged(path) => {
                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && ext == "md"
                    && let Some(filename) = path.file_name().and_then(|n| n.to_str())
                {
                    let filename = filename.to_string();
                    if let Ok(content) = self.note_store.load_note(&filename) {
                        let modified_at = self.note_store.get_modified_at(&filename).unwrap_or(0);
                        self.index
                            .add_note(filename.clone(), content.clone(), modified_at);

                        if Some(&filename) == self.editor.current_note.as_ref()
                            && !self.editor.has_unsaved_changes()
                        {
                            self.editor.set_content(&filename, &content);
                        }
                        self.update_search();
                    }
                }
            }
            Action::MouseDown(x, y) => {
                if !self.popup_stack.is_empty() {
                    self.popup_stack.clear();
                    return;
                }
                if self.show_about {
                    self.show_about = false;
                    return;
                }
                if self.show_settings {
                    self.show_settings = false;
                    return;
                }

                if self.is_on_divider(x, y) {
                    self.is_resizing_sidebar = true;
                    self.update_sidebar_width_percent(x, y);
                    return;
                }

                if let Some(update_area) = self.update_area
                    && x >= update_area.x
                    && x < update_area.x + update_area.width
                    && y >= update_area.y
                    && y < update_area.y + update_area.height
                {
                    self.update(Action::UpdateApp);
                    return;
                }

                if x >= self.search_area.x
                    && x < self.search_area.x + self.search_area.width
                    && y >= self.search_area.y
                    && y < self.search_area.y + self.search_area.height
                {
                    self.focus = Focus::SearchBar;
                } else if x >= self.list_area.x
                    && x < self.list_area.x + self.list_area.width
                    && y >= self.list_area.y
                    && y < self.list_area.y + self.list_area.height
                {
                    self.focus = Focus::NoteList;
                    if let Some(selected) =
                        self.note_list
                            .click_at(y, self.list_area, self.config.show_modified_time)
                    {
                        if !selected.is_empty() {
                            if let Ok(content) = self.note_store.load_note(&selected) {
                                self.editor.set_content(&selected, &content);
                            }
                        } else {
                            self.update(Action::SubmitSearch);
                        }
                    }
                } else if x >= self.editor_area.x
                    && x < self.editor_area.x + self.editor_area.width
                    && y >= self.editor_area.y
                    && y < self.editor_area.y + self.editor_area.height
                {
                    self.focus = Focus::Editor;
                }
            }
            Action::MouseDrag(x, y) => {
                if self.is_resizing_sidebar || (self.show_notes_list && self.is_on_divider(x, y)) {
                    self.is_resizing_sidebar = true;
                    self.update_sidebar_width_percent(x, y);
                }
            }
            Action::MouseUp(_, _) => {
                if self.is_resizing_sidebar {
                    self.is_resizing_sidebar = false;
                    let _ = self.config.save();
                }
            }
            Action::MouseClick(x, y) => {
                self.update(Action::MouseDown(x, y));
                self.update(Action::MouseUp(x, y));
            }
            Action::MouseScrollUp(x, y) => {
                let in_list = x >= self.list_area.x
                    && x < self.list_area.x + self.list_area.width
                    && y >= self.list_area.y
                    && y < self.list_area.y + self.list_area.height;
                let in_editor = x >= self.editor_area.x
                    && x < self.editor_area.x + self.editor_area.width
                    && y >= self.editor_area.y
                    && y < self.editor_area.y + self.editor_area.height;

                if in_list || (!in_list && !in_editor && self.focus == Focus::NoteList) {
                    self.note_list.previous();
                    self.note_list.previous();
                    self.note_list.previous();
                } else if in_editor || (!in_list && !in_editor && self.focus == Focus::Editor) {
                    for _ in 0..3 {
                        self.editor.handle_key(crossterm::event::KeyEvent::new(
                            KeyCode::Up,
                            crossterm::event::KeyModifiers::NONE,
                        ));
                    }
                }
            }
            Action::MouseScrollDown(x, y) => {
                let in_list = x >= self.list_area.x
                    && x < self.list_area.x + self.list_area.width
                    && y >= self.list_area.y
                    && y < self.list_area.y + self.list_area.height;
                let in_editor = x >= self.editor_area.x
                    && x < self.editor_area.x + self.editor_area.width
                    && y >= self.editor_area.y
                    && y < self.editor_area.y + self.editor_area.height;

                if in_list || (!in_list && !in_editor && self.focus == Focus::NoteList) {
                    self.note_list.next();
                    self.note_list.next();
                    self.note_list.next();
                } else if in_editor || (!in_list && !in_editor && self.focus == Focus::Editor) {
                    for _ in 0..3 {
                        self.editor.handle_key(crossterm::event::KeyEvent::new(
                            KeyCode::Down,
                            crossterm::event::KeyModifiers::NONE,
                        ));
                    }
                }
            }

            Action::Render | Action::Resize(_, _) => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let theme = crate::theme::ThemePalette::from_name(&self.config.theme);

// Render dynamic status bar (no background color, uses terminal default)
        let key_style = Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD);
        let text_style = Style::default().fg(Color::Reset);
        let sep_style = Style::default().fg(theme.border_inactive);

        let mut shortcut_groups = vec![
            vec![
                Span::styled(" [Tab/Esc] ", key_style),
                Span::styled("Switch Focus", text_style),
                Span::styled(" │", sep_style),
            ],
        ];

        let mut stats_span = None;

        if self.focus == Focus::Editor {
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+Z] ", key_style),
                Span::styled("Undo", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+Y] ", key_style),
                Span::styled("Redo", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+Q] ", key_style),
                Span::styled("Quit", text_style),
                Span::styled(" │", sep_style),
            ]);

            let (backlinks_count, modified_str) = if let Some(ref note) = self.editor.current_note {
                let count = self.index.get_backlinks(note).len();
                let mod_time = self
                    .note_store
                    .get_modified_at(note)
                    .map(format_timestamp)
                    .unwrap_or_default();
                (count, mod_time)
            } else {
                (0, String::new())
            };

            let stats = if modified_str.is_empty() {
                format!(
                    " Words: {} │ Chars: {} │ Backlinks: {} ",
                    self.editor.word_count(),
                    self.editor.char_count(),
                    backlinks_count
                )
            } else {
                format!(
                    " Modified: {} │ Words: {} │ Chars: {} │ Backlinks: {} ",
                    modified_str,
                    self.editor.word_count(),
                    self.editor.char_count(),
                    backlinks_count
                )
            };
            stats_span = Some(Span::styled(stats, Style::default().fg(Color::Yellow)));
        } else {
            shortcut_groups.push(vec![
                Span::styled(" [?] ", key_style),
                Span::styled("Help", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+V] ", key_style),
                Span::styled("About", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+B] ", key_style),
                Span::styled("Toggle List", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("Create/Edit", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+D] ", key_style),
                Span::styled("Delete", text_style),
                Span::styled(" │", sep_style),
            ]);
            shortcut_groups.push(vec![
                Span::styled(" [Ctrl+Q] ", key_style),
                Span::styled("Quit", text_style),
                Span::styled(" │", sep_style),
            ]);

            if let Some(selected) = self.note_list.selected_note()
                && let Some(ts) = self.note_store.get_modified_at(&selected)
            {
                let date_str = format_timestamp(ts);
                if !date_str.is_empty() {
                    stats_span = Some(Span::styled(
                        format!(" Modified: {date_str} "),
                        Style::default().fg(Color::Yellow),
                    ));
                }
            }
        }

        let available_width = frame.area().width as usize;
        let stats_width = stats_span.as_ref().map(|s| s.content.chars().count()).unwrap_or(0);
        let more_span_width = 16; // Length of " [Ctrl+E] More │"

        let mut primary_spans = Vec::new();
        let mut secondary_spans = Vec::new();
        let mut current_width = 0;
        let mut overflow = false;

        for group in shortcut_groups.clone() {
            let group_width: usize = group.iter().map(|s| s.content.chars().count()).sum();
            
            if !overflow && current_width + group_width + stats_width + more_span_width <= available_width {
                primary_spans.extend(group);
                current_width += group_width;
            } else {
                overflow = true;
                secondary_spans.extend(group);
            }
        }

        let mut final_spans = Vec::new();
        let mut status_height = 1;

        if overflow {
            if self.show_more_shortcuts {
                for group in shortcut_groups {
                    final_spans.extend(group);
                }
                final_spans.push(Span::styled(" [Ctrl+E] ", key_style));
                final_spans.push(Span::styled("Back", text_style));
                final_spans.push(Span::styled(" │", sep_style));
                
                let total_len: usize = final_spans.iter().map(|s| s.content.chars().count()).sum();
                let total_with_stats = total_len + stats_width;
                status_height = ((total_with_stats as u16 + frame.area().width - 1) / frame.area().width).max(1);
            } else {
                final_spans.extend(primary_spans);
                final_spans.push(Span::styled(" [Ctrl+E] ", key_style));
                final_spans.push(Span::styled("More", text_style));
                final_spans.push(Span::styled(" │", sep_style));
            }
        } else {
            for group in shortcut_groups {
                final_spans.extend(group);
            }
        }

        if let Some(last) = final_spans.last() {
            if last.content == " │" {
                final_spans.pop();
            }
        }

        if stats_span.is_some() && !final_spans.is_empty() {
            final_spans.push(Span::styled(" │", sep_style));
        }

        if let Some(stats) = stats_span {
            final_spans.push(stats);
        }

        let status_text = Line::from(final_spans);

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header bar (App Title & Version)
                Constraint::Length(3), // Search bar
                Constraint::Min(1),    // Main content
                Constraint::Length(status_height), // Status bar
            ])
            .split(frame.area());

        // Render top header bar (Title left, Version right)
        let header_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[0]);

        let title_p = Paragraph::new(Line::from(Span::styled(
            " ⚡ Ferronote",
            Style::default()
                .fg(theme.title)
                .add_modifier(Modifier::BOLD),
        )));
        let mut header_spans = vec![];
        if let Some(latest) = &self.latest_version {
            header_spans.push(Span::styled(
                format!("[Update to v{}] ", latest),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        header_spans.push(Span::styled(
            format!("v{} ", include_str!("../.version").trim()),
            Style::default().fg(Color::DarkGray),
        ));

        let version_p = Paragraph::new(Line::from(header_spans)).alignment(Alignment::Right);

        if let Some(latest) = &self.latest_version {
            // we will approximate the update area based on the right aligned text length
            let text_len = latest.len() + 15;
            let mut area = header_layout[1];
            area.x = area
                .x
                .saturating_add(area.width.saturating_sub(text_len as u16 + 10)); // approximate clickable zone
            self.update_area = Some(area);
        } else {
            self.update_area = None;
        }

        frame.render_widget(title_p, header_layout[0]);
        frame.render_widget(version_p, header_layout[1]);

        let sidebar_size = self.config.sidebar_width_percent.clamp(20, 40);
        let direction = if self.config.notes_list_position == "left" {
            Direction::Horizontal
        } else {
            Direction::Vertical
        };

        let content_layout = if self.show_notes_list {
            Layout::default()
                .direction(direction)
                .constraints([
                    Constraint::Percentage(sidebar_size),
                    Constraint::Percentage(100 - sidebar_size),
                ])
                .split(main_layout[2])
        } else {
            Layout::default()
                .direction(direction)
                .constraints([
                    Constraint::Length(0),
                    Constraint::Percentage(100),
                ])
                .split(main_layout[2])
        };

        self.search_area = main_layout[1];
        self.content_area = main_layout[2];
        self.list_area = content_layout[0];
        self.editor_area = content_layout[1];

        // Render components
        self.search_bar.draw(
            frame,
            main_layout[1],
            self.focus == Focus::SearchBar,
            &theme,
        );
        if self.show_notes_list {
            self.note_list.draw(
                frame,
                content_layout[0],
                self.focus == Focus::NoteList,
                self.config.show_modified_time,
                &self.config.default_sort,
                &theme,
            );
        }
        self.editor.draw(
            frame,
            content_layout[1],
            self.focus == Focus::Editor,
            self.config.word_wrap,
            &theme,
        );

        let status_bar = Paragraph::new(status_text)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(status_bar, main_layout[3]);

        // Render Delete Confirmation Overlay
        if self.show_delete_confirmation {
            let mut selected_note = self.note_list.selected_note().unwrap_or_default();
            if selected_note.chars().count() > 20 {
                selected_note = format!("{}...", selected_note.chars().take(17).collect::<String>());
            }
            
            let confirm_lines = vec![
                Line::from(vec![
                    Span::styled(" Are you sure you want to delete ", Style::default().fg(Color::White)),
                    Span::styled(format!("'{}'", selected_note), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled("? ", Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(Span::styled(" [Y]es / [N]o ", Style::default().fg(Color::DarkGray))),
            ];

            let confirm_block = Paragraph::new(confirm_lines)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Confirm Deletion ")
                        .border_style(Style::default().fg(Color::Red)),
                );

            let area = frame.area();
            let width = 60;
            let height = 5;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(confirm_block, popup_area);
        }

        // Render Rename Prompt Overlay
        if self.show_rename_prompt {
            let mut rename_block = self.rename_input.clone();
            rename_block.set_block(
                Block::default()
                    .title(" Rename Note (Enter to confirm, Esc to cancel) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_active)),
            );
            rename_block.set_style(Style::default().fg(theme.search_fg));
            rename_block.set_cursor_line_style(Style::default());

            let area = frame.area();
            let width = 60;
            let height = 3;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(&rename_block, popup_area);
        }

        // Render Popups (Help, etc.)
        use crate::components::DrawableComponent;
        let _ = self.popup_stack.draw(frame, frame.area());

        // Render About Overlay
        if self.show_about {
            let version = include_str!("../.version").trim();
            let about_lines = vec![
                Line::from(Span::styled(
                    format!(" ℹ️  Ferronote v{version}"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    " Notes at the speed of thought — Notational Velocity for TUI",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "  Creator   : ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("Tareq Mohammad Yousuf ", Style::default().fg(Color::White)),
                    Span::styled("(https://tareqmy.com)", Style::default().fg(Color::Blue)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Contact   : ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("tareq.y@gmail.com", Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  GitHub    : ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "https://github.com/tareqmy/ferronote",
                        Style::default().fg(Color::Blue),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  License   : ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("MIT License", Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    " [Press any key or Esc to close]",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let about_block = Paragraph::new(about_lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" About (Ctrl+V) ")
                    .border_style(Style::default().fg(Color::Yellow)),
            );

            let area = frame.area();
            let width = 62;
            let height = 13;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(about_block, popup_area);
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
                (
                    "Show Modified Time",
                    if self.config.show_modified_time {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    },
                ),
                (
                    "Word Wrap",
                    if self.config.word_wrap {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    },
                ),
                (
                    "Note List Position",
                    if self.config.notes_list_position == "top" {
                        "Top".to_string()
                    } else {
                        "Left".to_string()
                    },
                ),
                (
                    "Mouse Capture",
                    if self.config.mouse_capture {
                        "Enabled".to_string()
                    } else {
                        "Disabled (Native Text Select)".to_string()
                    },
                ),
                (
                    "External Editor",
                    if self.show_settings && self.settings_selected_index == 11 {
                        format!("{}█", self.config.external_editor)
                    } else if self.config.external_editor.is_empty() || self.config.external_editor == "auto" {
                        "Auto ($VISUAL/$EDITOR)".to_string()
                    } else {
                        self.config.external_editor.clone()
                    },
                ),
            ];

            let mut lines = vec![
                Line::from(Span::styled(
                    "                   ⚙️  Ferronote Settings",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (idx, (label, val)) in options_data.iter().enumerate() {
                let is_selected = idx == self.settings_selected_index;
                let indent = "     ";
                let prefix = if is_selected { " ▶ " } else { "   " };
                let label_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                };

                let sep_style = Style::default().fg(Color::DarkGray);

                let val_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                lines.push(Line::from(vec![
                    Span::raw(indent),
                    Span::styled(prefix, label_style),
                    Span::styled(format!("{:<22}", label), label_style),
                    Span::styled(": ", sep_style),
                    Span::styled(format!("[ {:<18} ]", val), val_style),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  [Up/Down] Select   [Left/Right/Enter] Modify   [Esc/Ctrl+P] Close",
                Style::default().fg(Color::DarkGray),
            )));

            let settings_block = Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Settings (Ctrl+P) "),
            );

            let area = frame.area();
            let width = 72;
            let height = 18;
            let x = (area.width.saturating_sub(width)) / 2;
            let y = (area.height.saturating_sub(height)) / 2;
            let popup_area = ratatui::layout::Rect::new(x, y, width, height);

            frame.render_widget(Clear, popup_area);
            frame.render_widget(settings_block, popup_area);
        }
    }

    fn trigger_update(&self) {
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_str = exe_path.to_string_lossy();

        let mut cmd = if exe_str.contains(".cargo/bin") || exe_str.contains(".cargo\\bin") {
            let mut c = std::process::Command::new("cargo");
            c.args(["install", "--git", "https://github.com/tareqmy/ferronote"]);
            c
        } else if exe_str.contains("homebrew/bin")
            || exe_str.contains("linuxbrew")
            || exe_str.contains("Cellar")
        {
            let mut c = std::process::Command::new("brew");
            c.args(["upgrade", "tareqmy/tap/ferronote"]);
            c
        } else if exe_str.contains(".local/bin") || exe_str.contains("AppData\\Local") {
            if cfg!(windows) {
                let mut c = std::process::Command::new("powershell");
                c.args(["-Command", "irm https://raw.githubusercontent.com/tareqmy/ferronote/master/install.ps1 | iex"]);
                c
            } else {
                let mut c = std::process::Command::new("sh");
                c.arg("-c").arg("curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/install.sh | sh");
                c
            }
        } else {
            // Let the package manager handle it
            return;
        };

        // Detach the child process so it survives app termination
        let _ = cmd.spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn setup_test_app() -> (App<'static>, tempfile::TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = NoteStore::new(temp_dir.path().to_path_buf()).unwrap();
        let app = App::new(store);
        (app, temp_dir)
    }

    #[test]
    fn test_app_initialization_and_create_note() {
        let (mut app, _temp_dir) = setup_test_app();
        assert!(!app.should_quit);
        assert_eq!(app.focus, Focus::SearchBar);
        // NoteStore creates default notes on empty directory
        assert_eq!(app.note_list.items.len(), 2);
        let titles: Vec<_> = app
            .note_list
            .items
            .iter()
            .map(|i| i.title.as_str())
            .collect();
        assert!(titles.contains(&"Welcome to Ferronote"));
        assert!(titles.contains(&"Lorem Ipsum"));

        let filename = app.create_note("First Note").unwrap();
        assert_eq!(filename, "First Note.md");
        assert!(app.note_store.filenames().contains(&filename));
        assert_eq!(app.note_list.items.len(), 3);
    }

    #[test]
    fn test_app_submit_search_creates_note() {
        let (mut app, _temp_dir) = setup_test_app();

        // Type query into search bar
        app.search_bar
            .handle_key(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::NONE));
        app.search_bar
            .handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        app.search_bar
            .handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE));

        app.update_search();
        assert!(app.note_list.items[0].is_create_prompt);

        // Submit search creates note and jumps to editor
        app.update(Action::SubmitSearch);
        assert_eq!(app.focus, Focus::Editor);
        assert_eq!(app.editor.current_note, Some("New.md".to_string()));
    }

    #[test]
    fn test_app_search_always_includes_create_entry_at_top() {
        let (mut app, _temp_dir) = setup_test_app();
        let _filename = app.create_note("ExistingNote").unwrap();

        // Search for phrase matching existing note
        app.search_bar.clear();
        for c in "ExistingNote".chars() {
            app.search_bar
                .handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        app.update_search();

        // Item 0 must be the create prompt
        assert!(app.note_list.items[0].is_create_prompt);
        assert_eq!(
            app.note_list.items[0].title,
            "Create new note: 'ExistingNote'"
        );

        // Item 1 must be the filtered matching note
        assert_eq!(app.note_list.items[1].title, "ExistingNote");
    }

    #[test]
    fn test_app_delete_note() {
        let (mut app, _temp_dir) = setup_test_app();
        let filename = app.create_note("To Delete").unwrap();
        assert!(app.note_store.filenames().contains(&filename));

        // Select the newly created note in the note_list
        if let Some(idx) = app
            .note_list
            .items
            .iter()
            .position(|r| r.filename == filename)
        {
            app.note_list.state.select(Some(idx));
        }

        app.update(Action::DeleteNote);
        assert!(!app.note_store.filenames().contains(&filename));
        assert_eq!(app.note_store.list_trash().unwrap().len(), 1);
    }

    #[test]
    fn test_app_settings_overlay_cycling() {
        let (mut app, _temp_dir) = setup_test_app();
        assert!(!app.show_settings);

        app.update(Action::ToggleSettings);
        assert!(app.show_settings);

        let initial_ext = app.config.default_extension.clone();
        app.update(Action::ChangeSettingOption(true));
        assert_ne!(app.config.default_extension, initial_ext);

        app.update(Action::NextSetting);
        assert_eq!(app.settings_selected_index, 1);

        app.update(Action::PrevSetting);
        assert_eq!(app.settings_selected_index, 0);

        // Navigate to Show Modified Time (option 7)
        for _ in 0..7 {
            app.update(Action::NextSetting);
        }
        assert_eq!(app.settings_selected_index, 7);
        let initial_mod_time = app.config.show_modified_time;
        app.update(Action::ChangeSettingOption(true));
        assert_eq!(app.config.show_modified_time, !initial_mod_time);

        // Navigate to Word Wrap (option 8)
        app.update(Action::NextSetting);
        assert_eq!(app.settings_selected_index, 8);
        let initial_word_wrap = app.config.word_wrap;
        app.update(Action::ChangeSettingOption(true));
        assert_eq!(app.config.word_wrap, !initial_word_wrap);

        // Navigate to Note List Position (option 9)
        app.update(Action::NextSetting);
        assert_eq!(app.settings_selected_index, 9);
        let initial_pos = app.config.notes_list_position.clone();
        let expected_next = if initial_pos == "top" { "left" } else { "top" };
        app.update(Action::ChangeSettingOption(true));
        assert_eq!(app.config.notes_list_position, expected_next);
        app.update(Action::ChangeSettingOption(true));
        assert_eq!(app.config.notes_list_position, initial_pos);

        // Test FirstSetting / LastSetting (Home / End)
        app.update(Action::FirstSetting);
        assert_eq!(app.settings_selected_index, 0);
        app.update(Action::LastSetting);
        assert_eq!(app.settings_selected_index, 11);

        // Test PageUp / PageDownSetting
        app.update(Action::PageUpSetting);
        assert_eq!(app.settings_selected_index, 6); // 11 - 5 = 6
        app.update(Action::PageUpSetting);
        assert_eq!(app.settings_selected_index, 1); // 6 - 5 = 1
        app.update(Action::PageUpSetting);
        assert_eq!(app.settings_selected_index, 0); // 1 - 5 = 0 (saturates)

        app.update(Action::PageDownSetting);
        assert_eq!(app.settings_selected_index, 5); // 0 + 5 = 5
        app.update(Action::PageDownSetting);
        assert_eq!(app.settings_selected_index, 10); // 5 + 5 = 10
        app.update(Action::PageDownSetting);
        assert_eq!(app.settings_selected_index, 11); // 10 + 5 = 15 (capped at 11)

        app.update(Action::ToggleSettings);
        assert!(!app.show_settings);
    }

    #[test]
    fn test_app_about_overlay_toggle() {
        let (mut app, _temp_dir) = setup_test_app();
        assert!(!app.show_about);

        app.update(Action::ToggleAbout);
        assert!(app.show_about);

        app.update(Action::ToggleAbout);
        assert!(!app.show_about);

        let action = app.handle_key(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL));
        assert_eq!(action, Some(Action::ToggleAbout));
    }

    #[test]
    fn test_app_mouse_click_panel_focus() {
        let (mut app, _temp_dir) = setup_test_app();
        app.search_area = Rect::new(0, 0, 80, 3);
        app.list_area = Rect::new(0, 3, 24, 20);
        app.editor_area = Rect::new(24, 3, 56, 20);

        // Click Search Bar area
        app.update(Action::MouseClick(10, 1));
        assert_eq!(app.focus, Focus::SearchBar);

        // Click Note List area
        app.update(Action::MouseClick(5, 5));
        assert_eq!(app.focus, Focus::NoteList);

        // Click Editor area
        app.update(Action::MouseClick(30, 10));
        assert_eq!(app.focus, Focus::Editor);
    }

    #[test]
    fn test_app_help_overlay_toggle() {
        let (mut app, _temp_dir) = setup_test_app();
        assert!(app.popup_stack.is_empty());

        app.update(Action::ToggleHelp);
        assert!(!app.popup_stack.is_empty());

        app.update(Action::ToggleHelp);
        assert!(app.popup_stack.is_empty());
    }

    #[test]
    fn test_app_editor_view_mode_by_default() {
        let (mut app, _temp_dir) = setup_test_app();
        let filename = app.create_note("ViewMode Test").unwrap();
        app.update(Action::SelectNote(Some(filename)));

        // Must be in View Mode by default
        assert!(!app.editor.is_editing);

        // Pressing 'e' enters Edit Mode
        app.editor
            .handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        assert!(app.editor.is_editing);

        // Pressing Esc exits Edit Mode back to View Mode
        app.editor
            .handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.editor.is_editing);
    }



    #[test]
    fn test_app_auto_save_on_tick() {
        let (mut app, _temp_dir) = setup_test_app();
        let filename = app.create_note("AutoSave Test").unwrap();
        app.update(Action::SelectNote(Some(filename.clone())));
        app.editor
            .set_content(&filename, "# AutoSave Test\nExisting content");

        // Clear any pending search debouncing from note creation
        app.last_search_input = None;

        // Simulate user typing in editor
        if let Some(ref note) = app.editor.current_note
            && let Some(ta) = app.editor.textareas.get_mut(note)
        {
            ta.insert_str("\nAppended edit.");
        }
        app.editor.last_edit_time = Some(Instant::now() - std::time::Duration::from_secs(2));
        assert!(app.editor.has_unsaved_changes());

        // Tick action should trigger auto-save
        app.update(Action::Tick);
        assert!(!app.editor.has_unsaved_changes());

        let disk_content = app.note_store.load_note(&filename).unwrap();
        assert!(disk_content.contains("Appended edit."));
    }


    #[test]
    fn test_app_keyboard_shortcuts() {
        let (mut app, _temp_dir) = setup_test_app();

        // Test Ctrl+S -> SaveNote
        let action = app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(action, Some(Action::SaveNote));

        // Test Ctrl+N -> New Note focus
        app.search_bar.textarea.insert_str("draft note");
        let action = app.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(action, Some(Action::SaveNote));
        assert_eq!(app.search_bar.query(), "");
        assert_eq!(app.focus, Focus::SearchBar);


        // Test Tab -> Cycle focus forward
        let action = app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(action, Some(Action::SaveNote));
        assert_eq!(app.focus, Focus::NoteList);

        // Test Shift+Tab (BackTab) -> Cycle focus backward
        let action = app.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
        assert_eq!(action, Some(Action::SaveNote));
        assert_eq!(app.focus, Focus::SearchBar);
    }

    #[test]
    fn test_mouse_drag_resize_sidebar_horizontal() {
        let (mut app, _temp_dir) = setup_test_app();
        app.config.notes_list_position = "left".to_string();
        app.config.sidebar_width_percent = 30;
        app.content_area = Rect::new(0, 4, 100, 30);
        app.list_area = Rect::new(0, 4, 30, 30);
        app.editor_area = Rect::new(30, 4, 70, 30);
        app.show_notes_list = true;

        // Verify divider detection around x = 30
        assert!(app.is_on_divider(30, 10));
        assert!(app.is_on_divider(29, 10));
        assert!(!app.is_on_divider(15, 10));

        // Press mouse down on divider (x = 30)
        app.update(Action::MouseDown(30, 10));
        assert!(app.is_resizing_sidebar);

        // Drag mouse to x = 38 (38% of 100)
        app.update(Action::MouseDrag(38, 10));
        assert_eq!(app.config.sidebar_width_percent, 38);

        // Release mouse button
        app.update(Action::MouseUp(38, 10));
        assert!(!app.is_resizing_sidebar);
    }

    #[test]
    fn test_mouse_drag_resize_sidebar_vertical() {
        let (mut app, _temp_dir) = setup_test_app();
        app.config.notes_list_position = "top".to_string();
        app.config.sidebar_width_percent = 30;
        app.content_area = Rect::new(0, 4, 100, 30);
        app.list_area = Rect::new(0, 4, 100, 9);
        app.editor_area = Rect::new(0, 13, 100, 21);
        app.show_notes_list = true;

        // Verify divider detection around y = 13
        assert!(app.is_on_divider(50, 13));
        assert!(app.is_on_divider(50, 12));
        assert!(!app.is_on_divider(50, 5));

        // Press mouse down on divider (y = 13)
        app.update(Action::MouseDown(50, 13));
        assert!(app.is_resizing_sidebar);

        // Drag mouse down to y = 16 (12 relative to 4, 12/30 = 40%)
        app.update(Action::MouseDrag(50, 16));
        assert_eq!(app.config.sidebar_width_percent, 40);

        // Release mouse button
        app.update(Action::MouseUp(50, 16));
        assert!(!app.is_resizing_sidebar);
    }

    #[test]
    fn test_mouse_drag_clamping() {
        let (mut app, _temp_dir) = setup_test_app();
        app.config.notes_list_position = "left".to_string();
        app.content_area = Rect::new(0, 4, 100, 30);
        app.list_area = Rect::new(0, 4, 30, 30);
        app.editor_area = Rect::new(30, 4, 70, 30);
        app.show_notes_list = true;

        // Drag way to the left (x = 2) -> clamped to 20%
        app.update(Action::MouseDown(30, 10));
        app.update(Action::MouseDrag(2, 10));
        assert_eq!(app.config.sidebar_width_percent, 20);

        // Drag way to the right (x = 95) -> clamped to 40%
        app.update(Action::MouseDrag(95, 10));
        assert_eq!(app.config.sidebar_width_percent, 40);

        app.update(Action::MouseUp(95, 10));
    }
}
