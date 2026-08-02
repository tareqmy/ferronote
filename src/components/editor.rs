use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
};
use std::collections::HashMap;
use std::time::Instant;
use tui_textarea::TextArea;

/// Main note editor component supporting Markdown editing, undo/redo, and wiki-link extraction.
#[derive(Debug, Clone)]
pub struct Editor<'a> {
    /// In-memory text areas per open note filename.
    pub textareas: HashMap<String, TextArea<'a>>,
    /// Active note filename being edited.
    pub current_note: Option<String>,
    /// Last edit timestamp used for debounced auto-save calculation.
    pub last_edit_time: Option<Instant>,
    /// Last saved content per note filename for dirty checking.
    pub original_content: HashMap<String, String>,
    /// Whether the editor is in active Edit mode (true) or View mode (false).
    pub is_editing: bool,
    /// Pending 'g' keypress for 'gg' normal mode vim shortcut.
    pub pending_g: bool,
    /// Pending 'd' keypress for 'dd' normal mode vim shortcut.
    pub pending_d: bool,
    /// Pending 'y' keypress for 'yy' normal mode vim shortcut.
    pub pending_y: bool,
    /// Pending count for 'yNy' / 'dNd' normal mode vim shortcuts.
    pub pending_y_count: usize,
    pub pending_d_count: usize,
    /// Pending 'v' keypress for 'Ctrl+v' insert literal mode.
    pub pending_v: bool,
    /// Active search textarea input. If Some, search mode is active.
    pub search_textarea: Option<TextArea<'a>>,
    pub current_search_query: HashMap<String, String>,
    /// Event queue for emitting app actions.
    pub queue: crate::queue::Queue,
}

// Default removed due to queue requirement

/// Soft wraps lines to fit max character width on space boundaries.
#[must_use]
pub fn wrap_text_to_width(lines: &[String], max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return lines.to_vec();
    }
    let mut result = Vec::new();
    for line in lines {
        if line.chars().count() <= max_width {
            result.push(line.clone());
        } else {
            let mut current_line = String::new();
            for word in line.split(' ') {
                if current_line.is_empty() {
                    current_line.push_str(word);
                } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    result.push(current_line);
                    current_line = word.to_string();
                }
            }
            if !current_line.is_empty() {
                result.push(current_line);
            }
        }
    }
    result
}

/// Maps an original line/column cursor position to a wrapped line/column position.
#[must_use]
pub fn map_cursor_to_wrapped(
    lines: &[String],
    orig_row: usize,
    orig_col: usize,
    max_width: usize,
) -> (u16, u16) {
    if max_width == 0 {
        return (orig_row as u16, orig_col as u16);
    }

    let mut current_wrapped_row: u16 = 0;

    for (r, line) in lines.iter().enumerate() {
        let is_target_row = r == orig_row;
        if line.chars().count() <= max_width {
            if is_target_row {
                return (
                    current_wrapped_row,
                    orig_col.min(line.chars().count()) as u16,
                );
            }
            current_wrapped_row += 1;
        } else {
            let words: Vec<&str> = line.split(' ').collect();
            let mut current_chunk = String::new();
            let mut chunk_start_char = 0;
            let mut char_count_in_orig = 0;

            for (w_idx, word) in words.iter().enumerate() {
                let add_len = if current_chunk.is_empty() {
                    word.chars().count()
                } else {
                    1 + word.chars().count()
                };

                if !current_chunk.is_empty() && current_chunk.chars().count() + add_len > max_width
                {
                    if is_target_row
                        && orig_col >= chunk_start_char
                        && orig_col < char_count_in_orig
                    {
                        let offset = orig_col - chunk_start_char;
                        return (current_wrapped_row, offset as u16);
                    }
                    current_wrapped_row += 1;
                    chunk_start_char = char_count_in_orig;
                    current_chunk = word.to_string();
                } else {
                    if !current_chunk.is_empty() {
                        current_chunk.push(' ');
                    }
                    current_chunk.push_str(word);
                }

                char_count_in_orig += word.chars().count();
                if w_idx < words.len() - 1 {
                    char_count_in_orig += 1;
                }
            }

            if !current_chunk.is_empty() {
                if is_target_row && orig_col >= chunk_start_char {
                    let offset = orig_col.saturating_sub(chunk_start_char);
                    let final_col = offset.min(current_chunk.chars().count());
                    return (current_wrapped_row, final_col as u16);
                }
                current_wrapped_row += 1;
            }
        }
    }

    (orig_row as u16, orig_col as u16)
}

impl Editor<'_> {
    /// Creates a new empty `Editor` component.
    #[must_use]
    pub fn new(queue: crate::queue::Queue) -> Self {
        Self {
            textareas: HashMap::new(),
            current_note: None,
            last_edit_time: None,
            original_content: HashMap::new(),
            is_editing: false,
            pending_g: false,
            pending_d: false,
            pending_y: false,
            pending_y_count: 0,
            pending_d_count: 0,
            pending_v: false,
            search_textarea: None,
            current_search_query: HashMap::new(),
            queue,
        }
    }

    pub fn toggle_edit_mode(&mut self) {
        self.is_editing = !self.is_editing;
    }

    pub fn set_content(&mut self, title: &str, content: &str) {
        self.pending_g = false;
        self.pending_d = false;
        self.pending_y = false;
        self.pending_y_count = 0;
        self.pending_d_count = 0;
        self.pending_v = false;
        if title.is_empty() {
            self.current_note = None;
            self.is_editing = false;
            return;
        }

        if self.current_note.as_deref() != Some(title) {
            self.is_editing = false;
        }
        self.current_note = Some(title.to_string());

        if !self.textareas.contains_key(title) {
            let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
            let ta = TextArea::new(lines);
            self.textareas.insert(title.to_string(), ta);
            self.original_content
                .insert(title.to_string(), content.to_string());
        } else {
            // Check if we need to reload due to external change (e.g. content doesn't match and no unsaved changes)
            if !self.has_unsaved_changes() && self.content() != content {
                let lines: Vec<String> = content.lines().map(ToString::to_string).collect();
                let ta = TextArea::new(lines);
                self.textareas.insert(title.to_string(), ta);
                self.original_content
                    .insert(title.to_string(), content.to_string());
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get_mut(note)
        {
            if let Some(ref mut search_ta) = self.search_textarea {
                match key.code {
                    KeyCode::Esc => {
                        self.search_textarea = None;
                    }
                    KeyCode::Enter => {
                        let query = search_ta.lines()[0].clone();
                        let pattern = format!("(?i){}", query);
                        let _ = ta.set_search_pattern(&pattern);
                        self.current_search_query.insert(note.clone(), pattern);
                        ta.search_forward(false);
                        self.search_textarea = None;
                    }
                    _ => {
                        search_ta.input(key);
                    }
                }
                return;
            }

            if !self.is_editing {
                let mut modified = false;

                if self.pending_y {
                    if let KeyCode::Char(c) = key.code {
                        if c.is_ascii_digit() {
                            let digit = c.to_digit(10).unwrap_or(0) as usize;
                            self.pending_y_count = self.pending_y_count.saturating_mul(10).saturating_add(digit);
                            return;
                        }
                    }
                }

                if self.pending_d {
                    if let KeyCode::Char(c) = key.code {
                        if c.is_ascii_digit() {
                            let digit = c.to_digit(10).unwrap_or(0) as usize;
                            self.pending_d_count = self.pending_d_count.saturating_mul(10).saturating_add(digit);
                            return;
                        }
                    }
                }

                if key.code == KeyCode::Char('g') {
                    self.pending_d = false;
                    self.pending_d_count = 0;
                    self.pending_y = false;
                    self.pending_y_count = 0;
                    if self.pending_g {
                        ta.move_cursor(tui_textarea::CursorMove::Top);
                        self.pending_g = false;
                    } else {
                        self.pending_g = true;
                    }
                    return;
                }
                
                let mut override_action = None;
                if self.pending_g && key.code == KeyCode::Char('e') {
                    override_action = Some(crate::shortcuts::EditorViewAction::WordEndBack);
                }
                self.pending_g = false;

                if key.code == KeyCode::Char('d') {
                    if self.pending_d {
                        let count = if self.pending_d_count > 0 { self.pending_d_count } else { 1 };
                        for _ in 0..count {
                            ta.move_cursor(tui_textarea::CursorMove::Head);
                            ta.delete_line_by_end();
                            if ta.cursor().0 == ta.lines().len() - 1 && ta.lines().len() > 1 {
                                ta.delete_char();
                            } else {
                                ta.delete_next_char();
                            }
                        }
                        self.pending_d = false;
                        self.pending_d_count = 0;
                        modified = true;
                    } else {
                        self.pending_d = true;
                        self.pending_d_count = 0;
                    }
                } else if key.code == KeyCode::Char('y') {
                    self.pending_d = false;
                    self.pending_d_count = 0;
                    if self.pending_y {
                        let count = if self.pending_y_count > 0 { self.pending_y_count } else { 1 };
                        let orig = ta.cursor();
                        ta.move_cursor(tui_textarea::CursorMove::Head);
                        ta.start_selection();
                        for _ in 0..count {
                            ta.move_cursor(tui_textarea::CursorMove::Down);
                        }
                        if ta.cursor().0 < orig.0 + count {
                            ta.move_cursor(tui_textarea::CursorMove::End);
                        }
                        ta.copy();
                        ta.cancel_selection();
                        ta.move_cursor(tui_textarea::CursorMove::Jump(orig.0 as u16, orig.1 as u16));
                        self.pending_y = false;
                        self.pending_y_count = 0;
                    } else {
                        self.pending_y = true;
                        self.pending_y_count = 0;
                    }
                } else {
                    self.pending_d = false;
                    self.pending_d_count = 0;
                    self.pending_y = false;
                    self.pending_y_count = 0;
                    
                    use crate::shortcuts::EditorViewAction;
                    let registry = crate::shortcuts::ShortcutRegistry::new();
                    if let Some(action) = override_action.or_else(|| registry.match_editor_view_shortcut(&key)) {
                        match action {
                            EditorViewAction::PasteAfter => {
                                let text = ta.yank_text();
                                let is_line_yank = text.ends_with('\n');
                                if is_line_yank {
                                    let old_row = ta.cursor().0;
                                    ta.move_cursor(tui_textarea::CursorMove::Down);
                                    if ta.cursor().0 == old_row {
                                        ta.move_cursor(tui_textarea::CursorMove::End);
                                        ta.insert_newline();
                                        ta.paste();
                                        ta.delete_char();
                                    } else {
                                        ta.move_cursor(tui_textarea::CursorMove::Head);
                                        ta.paste();
                                    }
                                } else {
                                    ta.move_cursor(tui_textarea::CursorMove::Forward);
                                    ta.paste();
                                    ta.move_cursor(tui_textarea::CursorMove::Back);
                                }
                                modified = true;
                            }
                            EditorViewAction::PasteBefore => {
                                let text = ta.yank_text();
                                let is_line_yank = text.ends_with('\n');
                                if is_line_yank {
                                    ta.move_cursor(tui_textarea::CursorMove::Head);
                                    ta.paste();
                                } else {
                                    ta.paste();
                                }
                                modified = true;
                            }
                            EditorViewAction::Undo => {
                                ta.undo();
                                modified = true;
                            }
                            EditorViewAction::Redo => {
                                ta.redo();
                                modified = true;
                            }
                            EditorViewAction::DeleteChar => {
                                let (row, col) = ta.cursor();
                                if col < ta.lines()[row].len() {
                                    ta.delete_next_char();
                                    modified = true;
                                }
                            }
                            EditorViewAction::SearchPrompt => {
                                let mut sta = TextArea::default();
                                sta.set_block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title("Search (Enter to search, Esc to cancel)"),
                                );
                                sta.set_cursor_line_style(Style::default());
                                self.search_textarea = Some(sta);
                                return;
                            }
                            EditorViewAction::SearchNext => {
                                ta.search_forward(false);
                            }
                            EditorViewAction::SearchPrev => {
                                ta.search_back(false);
                            }
                            EditorViewAction::EnterInsert => {
                                self.is_editing = true;
                            }
                            EditorViewAction::EnterInsertAppend => {
                                ta.move_cursor(tui_textarea::CursorMove::Forward);
                                self.is_editing = true;
                            }
                            EditorViewAction::EnterInsertHead => {
                                ta.move_cursor(tui_textarea::CursorMove::Head);
                                self.is_editing = true;
                            }
                            EditorViewAction::EnterInsertEnd => {
                                ta.move_cursor(tui_textarea::CursorMove::End);
                                self.is_editing = true;
                            }
                            EditorViewAction::EnterInsertOpenBelow => {
                                ta.move_cursor(tui_textarea::CursorMove::End);
                                ta.insert_newline();
                                self.is_editing = true;
                            }
                            EditorViewAction::EnterInsertOpenAbove => {
                                ta.move_cursor(tui_textarea::CursorMove::Head);
                                ta.insert_newline();
                                ta.move_cursor(tui_textarea::CursorMove::Up);
                                self.is_editing = true;
                            }
                            EditorViewAction::PageDown => {
                                for _ in 0..10 {
                                    ta.move_cursor(tui_textarea::CursorMove::Down);
                                }
                            }
                            EditorViewAction::PageUp => {
                                for _ in 0..10 {
                                    ta.move_cursor(tui_textarea::CursorMove::Up);
                                }
                            }
                            EditorViewAction::CursorUp => {
                                ta.move_cursor(tui_textarea::CursorMove::Up);
                            }
                            EditorViewAction::CursorDown => {
                                ta.move_cursor(tui_textarea::CursorMove::Down);
                            }
                            EditorViewAction::CursorLeft => {
                                ta.move_cursor(tui_textarea::CursorMove::Back);
                            }
                            EditorViewAction::CursorRight => {
                                ta.move_cursor(tui_textarea::CursorMove::Forward);
                            }
                            EditorViewAction::WordForward => {
                                ta.move_cursor(tui_textarea::CursorMove::WordForward);
                            }
                            EditorViewAction::WordBack => {
                                ta.move_cursor(tui_textarea::CursorMove::WordBack);
                            }
                            EditorViewAction::WordForwardWhitespace => {
                                let lines = ta.lines();
                                let (mut r, mut c) = ta.cursor();
                                if r < lines.len() {
                                    let mut current_line: Vec<char> = lines[r].chars().collect();
                                    let mut in_word = c < current_line.len() && !current_line[c].is_whitespace();
                                    loop {
                                        if c >= current_line.len() {
                                            r += 1;
                                            c = 0;
                                            if r >= lines.len() {
                                                r = lines.len() - 1;
                                                c = lines[r].chars().count();
                                                break;
                                            }
                                            current_line = lines[r].chars().collect();
                                            in_word = false;
                                        } else {
                                            let is_ws = current_line[c].is_whitespace();
                                            if in_word && is_ws {
                                                in_word = false;
                                            } else if !in_word && !is_ws {
                                                break;
                                            }
                                            c += 1;
                                        }
                                    }
                                    ta.move_cursor(tui_textarea::CursorMove::Jump(r as u16, c as u16));
                                }
                            }
                            EditorViewAction::WordBackWhitespace => {
                                let lines = ta.lines();
                                let (mut r, mut c) = ta.cursor();
                                if r < lines.len() {
                                    if c > 0 {
                                        c -= 1;
                                    } else if r > 0 {
                                        r -= 1;
                                        c = lines[r].chars().count();
                                        if c > 0 { c -= 1; }
                                    }
                                    
                                    let mut current_line: Vec<char> = lines[r].chars().collect();
                                    let mut in_word = c < current_line.len() && !current_line[c].is_whitespace();
                                    
                                    loop {
                                        if c == 0 && (current_line.is_empty() || current_line[0].is_whitespace() || r == 0) {
                                            if r == 0 {
                                                c = 0;
                                                break;
                                            }
                                            r -= 1;
                                            current_line = lines[r].chars().collect();
                                            c = current_line.len();
                                            if c > 0 { c -= 1; }
                                            in_word = false;
                                        } else {
                                            let is_ws = current_line[c].is_whitespace();
                                            if !in_word && !is_ws {
                                                in_word = true;
                                            } else if in_word && is_ws {
                                                c += 1;
                                                break;
                                            }
                                            if c > 0 {
                                                c -= 1;
                                            } else if r == 0 {
                                                c = 0;
                                                break;
                                            }
                                        }
                                    }
                                    ta.move_cursor(tui_textarea::CursorMove::Jump(r as u16, c as u16));
                                }
                            }
                            EditorViewAction::WordEndForward => {
                                let lines = ta.lines();
                                let (mut r, mut c) = ta.cursor();
                                if r < lines.len() {
                                    let mut current_line: Vec<char> = lines[r].chars().collect();
                                    if c < current_line.len() { c += 1; }
                                    let mut in_word = c < current_line.len() && !current_line[c].is_whitespace();
                                    
                                    loop {
                                        if c >= current_line.len() {
                                            if in_word {
                                                if c > 0 { c -= 1; }
                                                break;
                                            }
                                            r += 1;
                                            c = 0;
                                            if r >= lines.len() {
                                                r = lines.len() - 1;
                                                c = lines[r].chars().count();
                                                if c > 0 { c -= 1; }
                                                break;
                                            }
                                            current_line = lines[r].chars().collect();
                                            in_word = false;
                                        } else {
                                            let is_ws = current_line[c].is_whitespace();
                                            if !in_word && !is_ws {
                                                in_word = true;
                                            } else if in_word && is_ws {
                                                c -= 1;
                                                break;
                                            }
                                            c += 1;
                                        }
                                    }
                                    ta.move_cursor(tui_textarea::CursorMove::Jump(r as u16, c as u16));
                                }
                            }
                            EditorViewAction::WordEndBack => {
                                let lines = ta.lines();
                                let (mut r, mut c) = ta.cursor();
                                if r < lines.len() {
                                    if c > 0 {
                                        c -= 1;
                                    } else if r > 0 {
                                        r -= 1;
                                        c = lines[r].chars().count();
                                        if c > 0 { c -= 1; }
                                    }
                                    
                                    let mut current_line: Vec<char> = lines[r].chars().collect();
                                    let mut skipping_current = c < current_line.len() && !current_line[c].is_whitespace();
                                    
                                    loop {
                                        if c == 0 && (current_line.is_empty() || r == 0) {
                                            if r == 0 { c = 0; break; }
                                            r -= 1;
                                            current_line = lines[r].chars().collect();
                                            c = current_line.len();
                                            if c > 0 { c -= 1; }
                                        } else {
                                            let is_ws = current_line.get(c).map_or(true, |ch| ch.is_whitespace());
                                            if skipping_current {
                                                if is_ws { skipping_current = false; }
                                            } else {
                                                if !is_ws { break; }
                                            }
                                            if c > 0 {
                                                c -= 1;
                                            } else if r == 0 {
                                                c = 0;
                                                break;
                                            } else {
                                                r -= 1;
                                                current_line = lines[r].chars().collect();
                                                c = current_line.len();
                                                if c > 0 { c -= 1; }
                                            }
                                        }
                                    }
                                    ta.move_cursor(tui_textarea::CursorMove::Jump(r as u16, c as u16));
                                }
                            }
                            EditorViewAction::LineHead => {
                                ta.move_cursor(tui_textarea::CursorMove::Head);
                            }
                            EditorViewAction::LineFirstNonBlank => {
                                let lines = ta.lines();
                                let (r, _) = ta.cursor();
                                if r < lines.len() {
                                    let current_line = &lines[r];
                                    let mut new_col = 0;
                                    for (i, ch) in current_line.chars().enumerate() {
                                        if !ch.is_whitespace() {
                                            new_col = i;
                                            break;
                                        }
                                        new_col = i; // if all whitespace, go to end of whitespace
                                    }
                                    ta.move_cursor(tui_textarea::CursorMove::Jump(r as u16, new_col as u16));
                                }
                            }
                            EditorViewAction::LineEnd => {
                                ta.move_cursor(tui_textarea::CursorMove::End);
                            }
                            EditorViewAction::FileBottom => {
                                ta.move_cursor(tui_textarea::CursorMove::Bottom);
                            }
                            EditorViewAction::ClearSearch => {
                                let _ = ta.set_search_pattern("");
                                self.current_search_query.remove(note);
                            }
                            _ => {}
                        }
                    }
            }
                
                if modified {
                    self.last_edit_time = Some(std::time::Instant::now());
                }
                return;
            }

            if key.code == KeyCode::Esc || (!self.pending_v && key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)) {
                self.is_editing = false;
                return;
            }

            let modified = if self.pending_v {
                self.pending_v = false;
                if let KeyCode::Char(c) = key.code {
                    ta.insert_char(c);
                    true
                } else if key.code == KeyCode::Enter {
                    ta.insert_newline();
                    true
                } else if key.code == KeyCode::Tab {
                    ta.insert_str("    ");
                    true
                } else {
                    ta.input(key)
                }
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('v')
            {
                self.pending_v = true;
                false
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('z')
            {
                ta.undo()
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('y')
            {
                ta.redo()
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('w')
            {
                ta.delete_word()
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('t')
            {
                let (row, col) = ta.cursor();
                ta.move_cursor(tui_textarea::CursorMove::Head);
                ta.insert_str("    ");
                ta.move_cursor(tui_textarea::CursorMove::Jump(row as u16, (col + 4) as u16));
                true
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('d')
            {
                let (row, col) = ta.cursor();
                ta.move_cursor(tui_textarea::CursorMove::Head);
                let line = ta.lines()[row].clone();
                let mut spaces_to_remove = 0;
                for c in line.chars().take(4) {
                    if c == ' ' {
                        spaces_to_remove += 1;
                    } else {
                        break;
                    }
                }
                for _ in 0..spaces_to_remove {
                    ta.delete_next_char();
                }
                let new_col = col.saturating_sub(spaces_to_remove);
                ta.move_cursor(tui_textarea::CursorMove::Jump(row as u16, new_col as u16));
                spaces_to_remove > 0
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('u')
            {
                ta.delete_line_by_head()
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('h')
            {
                ta.delete_char()
            } else if key.code == KeyCode::PageDown {
                for _ in 0..10 {
                    ta.move_cursor(tui_textarea::CursorMove::Down);
                }
                true
            } else if key.code == KeyCode::PageUp {
                for _ in 0..10 {
                    ta.move_cursor(tui_textarea::CursorMove::Up);
                }
                true
            } else {
                ta.input(key)
            };

            if modified {
                self.last_edit_time = Some(Instant::now());
            }
        }
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        area: Rect,
        is_focused: bool,
        word_wrap: bool,
        theme: &crate::theme::ThemePalette,
    ) {
        let mode_str = if self.is_editing { " [EDIT] " } else { " [VIEW] " };
        let title = if let Some(ref note) = self.current_note {
            format!(
                " {}{} ",
                note.strip_suffix(".md").unwrap_or(note),
                mode_str
            )
        } else {
            " Editor ".to_string()
        };

        let border_color = if is_focused {
            if self.is_editing {
                theme.accent
            } else {
                theme.border_active
            }
        } else {
            theme.border_inactive
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(theme.title)));

        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get(note)
        {
            let mut ta_clone = if word_wrap {
                let max_width = (area.width as usize).saturating_sub(2);
                let wrapped_lines = wrap_text_to_width(ta.lines(), max_width);
                let (orig_row, orig_col) = ta.cursor();
                let (w_row, w_col) =
                    map_cursor_to_wrapped(ta.lines(), orig_row, orig_col, max_width);
                let mut wrapped_ta = TextArea::new(wrapped_lines);
                wrapped_ta.move_cursor(tui_textarea::CursorMove::Jump(w_row, w_col));
                wrapped_ta
            } else {
                ta.clone()
            };

            ta_clone.set_block(block);
            if let Some(query) = self.current_search_query.get(note) {
                let _ = ta_clone.set_search_pattern(query);
            }
            ta_clone.set_search_style(ratatui::style::Style::default().bg(ratatui::style::Color::Yellow).fg(ratatui::style::Color::Black));
            if is_focused {
                if self.is_editing {
                    ta_clone.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
                } else {
                    ta_clone.set_cursor_style(Style::default().bg(Color::DarkGray).fg(Color::White));
                }
            } else {
                ta_clone.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
            }
            frame.render_widget(&ta_clone, area);

            if let Some(ref search_ta) = self.search_textarea {
                let search_area = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Min(0),
                        ratatui::layout::Constraint::Length(3),
                    ])
                    .split(area)[1];
                frame.render_widget(ratatui::widgets::Clear, search_area);
                frame.render_widget(search_ta, search_area);
            }
            return;
        }

        // Empty state
        let mut empty_ta = TextArea::default();
        empty_ta.set_block(block);
        empty_ta.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
        frame.render_widget(&empty_ta, area);
    }

    #[must_use]
    pub fn content(&self) -> String {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get(note)
        {
            return ta.lines().join("\n");
        }
        String::new()
    }

    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        if let Some(ref note) = self.current_note
            && let Some(orig) = self.original_content.get(note)
        {
            return orig != &self.content();
        }
        false
    }

    pub fn mark_saved(&mut self) {
        if let Some(ref note) = self.current_note {
            self.original_content
                .insert(note.to_string(), self.content());
            self.last_edit_time = None;
        }
    }

    #[must_use]
    pub fn word_count(&self) -> usize {
        self.content().split_whitespace().count()
    }

    #[must_use]
    pub fn char_count(&self) -> usize {
        // Exclude newlines from char count to be more accurate to typical editors
        self.content()
            .chars()
            .filter(|c| !c.is_whitespace() || *c == ' ')
            .count()
    }

    pub fn extract_wiki_link_at_cursor(&self) -> Option<String> {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get(note)
        {
            let (row, col) = ta.cursor();
            if let Some(line) = ta.lines().get(row) {
                let mut current_idx = 0;
                while let Some(start) = line[current_idx..].find("[[") {
                    let start_idx = current_idx + start;
                    if let Some(end) = line[start_idx..].find("]]") {
                        let end_idx = start_idx + end + 2;
                        if col >= start_idx && col <= end_idx {
                            let link = &line[start_idx + 2..end_idx - 2];
                            return Some(link.to_string());
                        }
                        current_idx = end_idx;
                    } else {
                        break;
                    }
                }
            }
        }
        None
    }

    pub fn extract_url_at_cursor(&self) -> Option<String> {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get(note)
        {
            let (row, col) = ta.cursor();
            if let Some(line) = ta.lines().get(row) {
                let chars: Vec<char> = line.chars().collect();
                if col >= chars.len() || chars[col].is_whitespace() {
                    return None;
                }
                
                let mut start_idx = col;
                let mut end_idx = col;
                
                while start_idx > 0 && !chars[start_idx - 1].is_whitespace() {
                    start_idx -= 1;
                }
                while end_idx < chars.len() && !chars[end_idx].is_whitespace() {
                    end_idx += 1;
                }
                
                let word: String = chars[start_idx..end_idx].iter().collect();
                if word.starts_with("http://") || word.starts_with("https://") {
                    // strip common trailing punctuation that might be caught
                    let trimmed = word.trim_end_matches(&[',', '.', ';', '?', '!', ')', ']'][..]);
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }
}

impl crate::components::Component for Editor<'_> {
    fn event(&mut self, ev: &crate::event::Event) -> color_eyre::Result<crate::components::EventState> {
        if let crate::event::Event::Key(key) = ev {
            // Only consume events if the editor is in editing mode, 
            // or if we want it to handle view mode scrolling. 
            // We just proxy to handle_key for now.
            self.handle_key(*key);
            return Ok(crate::components::EventState::Consumed);
        }
        Ok(crate::components::EventState::NotConsumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_wiki_link_extraction() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        editor.set_content("test.md", "Check out [[Project Ideas]] for details.");

        // Place cursor inside the brackets
        if let Some(ref note) = editor.current_note
            && let Some(ta) = editor.textareas.get_mut(note)
        {
            ta.move_cursor(tui_textarea::CursorMove::WordForward);
            ta.move_cursor(tui_textarea::CursorMove::WordForward);
        }

        let link = editor.extract_wiki_link_at_cursor();
        assert_eq!(link, Some("Project Ideas".to_string()));
    }

    #[test]
    fn test_editor_debounced_save() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        assert_eq!(editor.current_note, None);
        assert_eq!(editor.word_count(), 0);
        assert_eq!(editor.char_count(), 0);
        assert!(!editor.has_unsaved_changes());

        editor.set_content("note.md", "Hello world!");
        assert_eq!(editor.current_note, Some("note.md".to_string()));
        assert_eq!(editor.word_count(), 2);
        assert_eq!(editor.char_count(), 12);
        assert!(!editor.has_unsaved_changes());

        // Reload with different content simulates edit
        if let Some(ref note) = editor.current_note
            && let Some(orig) = editor.original_content.get_mut(note)
        {
            *orig = "Different initial".to_string();
        }
        assert!(editor.has_unsaved_changes());

        editor.mark_saved();
        assert!(!editor.has_unsaved_changes());
    }

    #[test]
    fn test_editor_wrap_text_to_width() {
        let lines = vec![
            "Short line".to_string(),
            "This is a longer line that should be wrapped across multiple lines".to_string(),
        ];
        let wrapped = wrap_text_to_width(&lines, 20);
        assert_eq!(wrapped[0], "Short line");
        assert!(wrapped.len() > 2);
        for line in &wrapped {
            assert!(line.chars().count() <= 20);
        }
    }

    #[test]
    fn test_map_cursor_to_wrapped() {
        let lines = vec![
            "Short line".to_string(),
            "This is a longer line wrapped".to_string(),
        ];
        let (row, col) = map_cursor_to_wrapped(&lines, 0, 5, 20);
        assert_eq!((row, col), (0, 5));

        let (row2, _col2) = map_cursor_to_wrapped(&lines, 1, 22, 20);
        assert_eq!(row2, 2);
    }

    #[test]
    fn test_editor_dirty_checking() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        let content = (0..30)
            .map(|i| format!("Line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        editor.set_content("long.md", &content);

        // Initial cursor at (0, 0)
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor(), (0, 0));
        }

        // Send PageDown
        editor.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor(), (10, 0));
        }

        // Send PageUp
        editor.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor(), (0, 0));
        }
    }

    #[test]
    fn test_editor_mode_switching() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        editor.set_content("note.md", "Line 1\nLine 2");

        assert!(!editor.is_editing);

        // Enter edit mode via 'i'
        editor.handle_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        assert!(editor.is_editing);

        // Exit edit mode via Esc
        editor.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!editor.is_editing);

        // Enter edit mode via 'a'
        editor.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert!(editor.is_editing);
        editor.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!editor.is_editing);

        // Enter edit mode via 'o' (insert newline below)
        editor.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        assert!(editor.is_editing);
        let content = editor.content();
        assert!(content.contains("\n\n") || content.lines().count() == 3);
    }

    #[test]
    fn test_editor_vim_navigation() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        editor.set_content("note.md", "First word second word\nSecond line here");

        // View mode movement keys: w, b, 0, $, gg, G
        editor.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert!(ta.cursor().1 > 0);
        }

        editor.handle_key(KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor().1, 22);
        }

        editor.handle_key(KeyEvent::new(KeyCode::Char('0'), KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor().1, 0);
        }

        // G moves to bottom
        editor.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor().0, 1);
        }

        // gg moves to top
        editor.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        editor.handle_key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        if let Some(ref note) = editor.current_note {
            let ta = editor.textareas.get(note).unwrap();
            assert_eq!(ta.cursor().0, 0);
        }
    }

    #[test]
    fn test_editor_vim_search_prompt() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        editor.set_content("note.md", "apple banana cherry apple");

        // Activate search via '/'
        editor.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE));
        assert!(editor.search_textarea.is_some());

        // Type "apple"
        for c in "apple".chars() {
            editor.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }

        // Press Enter to perform search
        editor.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(editor.search_textarea.is_none());
        assert!(editor.current_search_query.contains_key("note.md"));

        // Press 'n' to jump to next match
        editor.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        // Press Esc to clear search highlight
        editor.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!editor.current_search_query.contains_key("note.md"));
    }

    #[test]
    fn test_editor_line_deletion_and_yank() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        editor.set_content("note.md", "Line 1\nLine 2\nLine 3");

        // dd deletes current line
        editor.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        editor.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        assert!(!editor.content().starts_with("Line 1"));

        // x deletes single character under cursor
        let old_len = editor.content().len();
        editor.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        assert_eq!(editor.content().len(), old_len - 1);
    }

    #[test]
    fn test_editor_yank_multiple_lines_y_ny() {
        let queue = crate::queue::Queue::new();
        let mut editor = Editor::new(queue);
        editor.set_content("note.md", "Line 1\nLine 2\nLine 3\nLine 4");

        // Execute y2y to yank 2 lines starting from Line 1
        editor.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));
        editor.handle_key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
        editor.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));

        // Move to bottom line (Line 4)
        editor.handle_key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));

        // Paste yanked text with 'p'
        editor.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));

        let content = editor.content();
        assert!(content.contains("Line 1\nLine 2"));
    }
}
