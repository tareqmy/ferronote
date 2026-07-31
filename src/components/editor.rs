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
}

impl Default for Editor<'_> {
    fn default() -> Self {
        Self::new()
    }
}

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
    pub fn new() -> Self {
        Self {
            textareas: HashMap::new(),
            current_note: None,
            last_edit_time: None,
            original_content: HashMap::new(),
            is_editing: false,
        }
    }

    pub fn toggle_edit_mode(&mut self) {
        self.is_editing = !self.is_editing;
    }

    pub fn set_content(&mut self, title: &str, content: &str) {
        if title.is_empty() {
            self.current_note = None;
            self.is_editing = false;
            return;
        }

        self.current_note = Some(title.to_string());
        self.is_editing = false;

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
            if !self.is_editing {
                match key.code {
                    KeyCode::Char('e') | KeyCode::Char('i') | KeyCode::Enter => {
                        self.is_editing = true;
                    }
                    KeyCode::PageDown => {
                        for _ in 0..10 {
                            ta.move_cursor(tui_textarea::CursorMove::Down);
                        }
                    }
                    KeyCode::PageUp => {
                        for _ in 0..10 {
                            ta.move_cursor(tui_textarea::CursorMove::Up);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        ta.move_cursor(tui_textarea::CursorMove::Up);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        ta.move_cursor(tui_textarea::CursorMove::Down);
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        ta.move_cursor(tui_textarea::CursorMove::Back);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        ta.move_cursor(tui_textarea::CursorMove::Forward);
                    }
                    KeyCode::Home => {
                        ta.move_cursor(tui_textarea::CursorMove::Head);
                    }
                    KeyCode::End => {
                        ta.move_cursor(tui_textarea::CursorMove::End);
                    }
                    _ => {}
                }
                return;
            }

            if key.code == KeyCode::Esc {
                self.is_editing = false;
                return;
            }

            let modified = if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('z')
            {
                ta.undo()
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('y')
            {
                ta.redo()
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
            theme.border_active
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
            if is_focused && self.is_editing {
                ta_clone.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
            } else {
                ta_clone.set_cursor_style(Style::default().bg(Color::Reset).fg(Color::Reset));
            }
            frame.render_widget(&ta_clone, area);
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

    pub fn format_bold(&mut self) {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get_mut(note)
        {
            let (row, col) = ta.cursor();
            let (new_lines, (new_row, new_col)) = toggle_bold_at(ta.lines(), row, col);
            let mut new_ta = TextArea::new(new_lines);
            new_ta.move_cursor(tui_textarea::CursorMove::Jump(new_row as u16, new_col as u16));
            *ta = new_ta;
            self.last_edit_time = Some(Instant::now());
        }
    }

    pub fn format_italic(&mut self) {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get_mut(note)
        {
            let (row, col) = ta.cursor();
            let (new_lines, (new_row, new_col)) = toggle_italic_at(ta.lines(), row, col);
            let mut new_ta = TextArea::new(new_lines);
            new_ta.move_cursor(tui_textarea::CursorMove::Jump(new_row as u16, new_col as u16));
            *ta = new_ta;
            self.last_edit_time = Some(Instant::now());
        }
    }

    pub fn indent_line(&mut self, tab_size: usize) {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get_mut(note)
        {
            let (row, col) = ta.cursor();
            let (new_lines, (new_row, new_col)) = indent_line_at(ta.lines(), row, col, tab_size);
            let mut new_ta = TextArea::new(new_lines);
            new_ta.move_cursor(tui_textarea::CursorMove::Jump(new_row as u16, new_col as u16));
            *ta = new_ta;
            self.last_edit_time = Some(Instant::now());
        }
    }

    pub fn unindent_line(&mut self, tab_size: usize) {
        if let Some(ref note) = self.current_note
            && let Some(ta) = self.textareas.get_mut(note)
        {
            let (row, col) = ta.cursor();
            let (new_lines, (new_row, new_col)) = unindent_line_at(ta.lines(), row, col, tab_size);
            let mut new_ta = TextArea::new(new_lines);
            new_ta.move_cursor(tui_textarea::CursorMove::Jump(new_row as u16, new_col as u16));
            *ta = new_ta;
            self.last_edit_time = Some(Instant::now());
        }
    }
}

/// Helper function to toggle bold formatting at given line and column.
#[must_use]
pub fn toggle_bold_at(lines: &[String], row: usize, col: usize) -> (Vec<String>, (usize, usize)) {
    let mut new_lines: Vec<String> = lines.to_vec();
    if lines.is_empty() {
        new_lines.push("****".to_string());
        return (new_lines, (0, 2));
    }
    let target_row = row.min(lines.len() - 1);
    let line = &lines[target_row];
    let chars: Vec<char> = line.chars().collect();

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    let mut word_start = None;
    let mut word_end = None;

    if col < chars.len() && is_word_char(chars[col]) {
        let mut s = col;
        while s > 0 && is_word_char(chars[s - 1]) {
            s -= 1;
        }
        let mut e = col;
        while e < chars.len() && is_word_char(chars[e]) {
            e += 1;
        }
        word_start = Some(s);
        word_end = Some(e);
    } else if col > 0 && (col - 1) < chars.len() && is_word_char(chars[col - 1]) {
        let mut s = col - 1;
        while s > 0 && is_word_char(chars[s - 1]) {
            s -= 1;
        }
        let mut e = col;
        while e < chars.len() && is_word_char(chars[e]) {
            e += 1;
        }
        word_start = Some(s);
        word_end = Some(e);
    }

    if let (Some(start), Some(end)) = (word_start, word_end) {
        let is_already_bold = start >= 2
            && end + 2 <= chars.len()
            && chars[start - 2] == '*'
            && chars[start - 1] == '*'
            && chars[end] == '*'
            && chars[end + 1] == '*';

        if is_already_bold {
            let prefix: String = chars[..start - 2].iter().collect();
            let word: String = chars[start..end].iter().collect();
            let suffix: String = chars[end + 2..].iter().collect();
            new_lines[target_row] = format!("{prefix}{word}{suffix}");
            let new_col = col.saturating_sub(2).max(start - 2);
            (new_lines, (target_row, new_col))
        } else {
            let prefix: String = chars[..start].iter().collect();
            let word: String = chars[start..end].iter().collect();
            let suffix: String = chars[end..].iter().collect();
            new_lines[target_row] = format!("{prefix}**{word}**{suffix}");
            let new_col = col + 2;
            (new_lines, (target_row, new_col))
        }
    } else {
        let insert_col = col.min(chars.len());
        let prefix: String = chars[..insert_col].iter().collect();
        let suffix: String = chars[insert_col..].iter().collect();
        new_lines[target_row] = format!("{prefix}****{suffix}");
        (new_lines, (target_row, insert_col + 2))
    }
}

/// Helper function to toggle italic formatting at given line and column.
#[must_use]
pub fn toggle_italic_at(
    lines: &[String],
    row: usize,
    col: usize,
) -> (Vec<String>, (usize, usize)) {
    let mut new_lines: Vec<String> = lines.to_vec();
    if lines.is_empty() {
        new_lines.push("**".to_string());
        return (new_lines, (0, 1));
    }
    let target_row = row.min(lines.len() - 1);
    let line = &lines[target_row];
    let chars: Vec<char> = line.chars().collect();

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    let mut word_start = None;
    let mut word_end = None;

    if col < chars.len() && is_word_char(chars[col]) {
        let mut s = col;
        while s > 0 && is_word_char(chars[s - 1]) {
            s -= 1;
        }
        let mut e = col;
        while e < chars.len() && is_word_char(chars[e]) {
            e += 1;
        }
        word_start = Some(s);
        word_end = Some(e);
    } else if col > 0 && (col - 1) < chars.len() && is_word_char(chars[col - 1]) {
        let mut s = col - 1;
        while s > 0 && is_word_char(chars[s - 1]) {
            s -= 1;
        }
        let mut e = col;
        while e < chars.len() && is_word_char(chars[e]) {
            e += 1;
        }
        word_start = Some(s);
        word_end = Some(e);
    }

    if let (Some(start), Some(end)) = (word_start, word_end) {
        let is_already_italic = start >= 1
            && end + 1 <= chars.len()
            && chars[start - 1] == '*'
            && chars[end] == '*'
            && (start < 2 || chars[start - 2] != '*')
            && (end + 1 >= chars.len() || chars[end + 1] != '*');

        if is_already_italic {
            let prefix: String = chars[..start - 1].iter().collect();
            let word: String = chars[start..end].iter().collect();
            let suffix: String = chars[end + 1..].iter().collect();
            new_lines[target_row] = format!("{prefix}{word}{suffix}");
            let new_col = col.saturating_sub(1).max(start - 1);
            (new_lines, (target_row, new_col))
        } else {
            let prefix: String = chars[..start].iter().collect();
            let word: String = chars[start..end].iter().collect();
            let suffix: String = chars[end..].iter().collect();
            new_lines[target_row] = format!("{prefix}*{word}*{suffix}");
            let new_col = col + 1;
            (new_lines, (target_row, new_col))
        }
    } else {
        let insert_col = col.min(chars.len());
        let prefix: String = chars[..insert_col].iter().collect();
        let suffix: String = chars[insert_col..].iter().collect();
        new_lines[target_row] = format!("{prefix}**{suffix}");
        (new_lines, (target_row, insert_col + 1))
    }
}

/// Helper function to indent line at given row with spaces.
#[must_use]
pub fn indent_line_at(
    lines: &[String],
    row: usize,
    col: usize,
    tab_size: usize,
) -> (Vec<String>, (usize, usize)) {
    let mut new_lines: Vec<String> = lines.to_vec();
    if lines.is_empty() {
        new_lines.push(" ".repeat(tab_size));
        return (new_lines, (0, tab_size));
    }
    let target_row = row.min(lines.len() - 1);
    let line = &lines[target_row];
    let spaces = " ".repeat(tab_size);
    new_lines[target_row] = format!("{spaces}{line}");
    (new_lines, (target_row, col + tab_size))
}

/// Helper function to unindent line at given row.
#[must_use]
pub fn unindent_line_at(
    lines: &[String],
    row: usize,
    col: usize,
    tab_size: usize,
) -> (Vec<String>, (usize, usize)) {
    let mut new_lines: Vec<String> = lines.to_vec();
    if lines.is_empty() {
        return (new_lines, (0, 0));
    }
    let target_row = row.min(lines.len() - 1);
    let line = &lines[target_row];

    if line.starts_with('\t') {
        new_lines[target_row] = line[1..].to_string();
        let new_col = col.saturating_sub(1);
        (new_lines, (target_row, new_col))
    } else {
        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
        if leading_spaces > 0 {
            let remove_count = leading_spaces.min(tab_size);
            new_lines[target_row] = line[remove_count..].to_string();
            let new_col = col.saturating_sub(remove_count);
            (new_lines, (target_row, new_col))
        } else {
            (new_lines, (target_row, col))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_link_at_cursor() {
        let mut editor = Editor::new();
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
    fn test_editor_stats_and_save_state() {
        let mut editor = Editor::new();
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
    fn test_editor_page_up_page_down() {
        let mut editor = Editor::new();
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
}
