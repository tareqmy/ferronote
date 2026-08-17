//! Vim interpreter for the editor's View (normal) mode.
//!
//! Owns the multi-key state machine (operators, counts, `g` prefixes) and the
//! motion/operator implementations that act on a [`TextArea`]. The editor
//! forwards View-mode key events here and applies the returned [`ViewOutcome`]
//! side effects (entering Insert mode, opening the search prompt, etc.).

use crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::{CursorMove, TextArea};

use crate::shortcuts::EditorViewAction;

/// Operator awaiting a motion or doubled key (`dd`, `yy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
}

/// An in-line character search kind (`f`/`t`/`F`/`T`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharSearch {
    /// `f`/`t` search forward; `F`/`T` search backward.
    pub forward: bool,
    /// `t`/`T` stop one character short of the target.
    pub till: bool,
}

/// Pending multi-key input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pending {
    #[default]
    None,
    /// `g` pressed, awaiting `g` (go to top) or `e` (word end backward).
    G,
    /// Operator pressed (`d`/`y`), awaiting count digits or a motion.
    Op { op: Operator, count: usize },
    /// `r` pressed, awaiting the replacement character.
    Replace,
    /// `f`/`t`/`F`/`T` pressed, awaiting the target character.
    Find(CharSearch),
    /// `Ctrl+V` pressed in Edit mode; the next key is inserted literally.
    Literal,
}

/// Vim interpreter state persisted on the editor across key events.
#[derive(Debug, Clone, Default)]
pub struct VimState {
    pub pending: Pending,
    /// Last `f`/`t`/`F`/`T` search, repeated by `;` and reversed by `,`.
    pub last_find: Option<(CharSearch, char)>,
}

impl VimState {
    /// Clears all pending multi-key state (e.g. when switching notes).
    pub fn reset(&mut self) {
        self.pending = Pending::None;
    }
}

/// Editor-level side effect requested by a View-mode key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
    None,
    EnterInsert,
    OpenSearchPrompt,
    ClearSearch,
}

/// Result of interpreting one View-mode key event.
#[derive(Debug, Clone, Copy)]
pub struct ViewOutcome {
    /// Whether the buffer content was modified (schedules auto-save).
    pub modified: bool,
    pub effect: SideEffect,
}

impl ViewOutcome {
    const fn none() -> Self {
        Self {
            modified: false,
            effect: SideEffect::None,
        }
    }
}

/// Interprets a key event in View mode, mutating the textarea and vim state.
pub fn handle_view_key(state: &mut VimState, ta: &mut TextArea, key: &KeyEvent) -> ViewOutcome {
    let mut out = ViewOutcome::none();

    // `r` + any character replaces the character under the cursor.
    if state.pending == Pending::Replace {
        state.pending = Pending::None;
        if let KeyCode::Char(c) = key.code {
            out.modified = replace_char(ta, c);
        }
        return out;
    }

    // `f`/`t`/`F`/`T` + any character searches for it in the current line.
    if let Pending::Find(search) = state.pending {
        state.pending = Pending::None;
        if let KeyCode::Char(target) = key.code {
            char_search(ta, search, target, false);
            state.last_find = Some((search, target));
        }
        return out;
    }

    // Count digits while an operator is pending (`d3d`, `y10y`).
    if let Pending::Op { op, count } = state.pending
        && let KeyCode::Char(c) = key.code
        && c.is_ascii_digit()
    {
        let digit = c.to_digit(10).unwrap_or(0) as usize;
        state.pending = Pending::Op {
            op,
            count: count.saturating_mul(10).saturating_add(digit),
        };
        return out;
    }

    if key.code == KeyCode::Char('g') {
        if state.pending == Pending::G {
            ta.move_cursor(CursorMove::Top);
            state.pending = Pending::None;
        } else {
            state.pending = Pending::G;
        }
        return out;
    }

    let mut override_action = None;
    if state.pending == Pending::G && key.code == KeyCode::Char('e') {
        override_action = Some(EditorViewAction::WordEndBack);
    }
    let prev_pending = state.pending;
    if state.pending == Pending::G {
        state.pending = Pending::None;
    }

    if key.code == KeyCode::Char('d') {
        if let Pending::Op {
            op: Operator::Delete,
            count,
        } = prev_pending
        {
            delete_lines(ta, if count > 0 { count } else { 1 });
            state.pending = Pending::None;
            out.modified = true;
        } else {
            state.pending = Pending::Op {
                op: Operator::Delete,
                count: 0,
            };
        }
        return out;
    }

    if key.code == KeyCode::Char('y') {
        if let Pending::Op {
            op: Operator::Yank,
            count,
        } = prev_pending
        {
            yank_lines(ta, if count > 0 { count } else { 1 });
            state.pending = Pending::None;
        } else {
            state.pending = Pending::Op {
                op: Operator::Yank,
                count: 0,
            };
        }
        return out;
    }

    state.pending = Pending::None;

    let registry = crate::shortcuts::ShortcutRegistry::new();
    let Some(action) = override_action.or_else(|| registry.match_editor_view_shortcut(key)) else {
        return out;
    };

    match action {
        EditorViewAction::PasteAfter => {
            paste_after(ta);
            out.modified = true;
        }
        EditorViewAction::PasteBefore => {
            paste_before(ta);
            out.modified = true;
        }
        EditorViewAction::Undo => {
            ta.undo();
            out.modified = true;
        }
        EditorViewAction::Redo => {
            ta.redo();
            out.modified = true;
        }
        EditorViewAction::DeleteChar => {
            let (row, col) = ta.cursor();
            if col < ta.lines()[row].len() {
                ta.delete_next_char();
                out.modified = true;
            }
        }
        EditorViewAction::SearchPrompt => {
            out.effect = SideEffect::OpenSearchPrompt;
        }
        EditorViewAction::SearchNext => {
            ta.search_forward(false);
        }
        EditorViewAction::SearchPrev => {
            ta.search_back(false);
        }
        EditorViewAction::EnterInsert => {
            out.effect = SideEffect::EnterInsert;
        }
        EditorViewAction::EnterInsertAppend => {
            ta.move_cursor(CursorMove::Forward);
            out.effect = SideEffect::EnterInsert;
        }
        EditorViewAction::EnterInsertHead => {
            ta.move_cursor(CursorMove::Head);
            out.effect = SideEffect::EnterInsert;
        }
        EditorViewAction::EnterInsertEnd => {
            ta.move_cursor(CursorMove::End);
            out.effect = SideEffect::EnterInsert;
        }
        EditorViewAction::EnterInsertOpenBelow => {
            ta.move_cursor(CursorMove::End);
            ta.insert_newline();
            out.effect = SideEffect::EnterInsert;
        }
        EditorViewAction::EnterInsertOpenAbove => {
            ta.move_cursor(CursorMove::Head);
            ta.insert_newline();
            ta.move_cursor(CursorMove::Up);
            out.effect = SideEffect::EnterInsert;
        }
        EditorViewAction::PageDown => {
            for _ in 0..10 {
                ta.move_cursor(CursorMove::Down);
            }
        }
        EditorViewAction::PageUp => {
            for _ in 0..10 {
                ta.move_cursor(CursorMove::Up);
            }
        }
        EditorViewAction::CursorUp => ta.move_cursor(CursorMove::Up),
        EditorViewAction::CursorDown => ta.move_cursor(CursorMove::Down),
        EditorViewAction::CursorLeft => ta.move_cursor(CursorMove::Back),
        EditorViewAction::CursorRight => ta.move_cursor(CursorMove::Forward),
        EditorViewAction::WordForward => ta.move_cursor(CursorMove::WordForward),
        EditorViewAction::WordBack => ta.move_cursor(CursorMove::WordBack),
        EditorViewAction::WordForwardWhitespace => word_forward_whitespace(ta),
        EditorViewAction::WordBackWhitespace => word_back_whitespace(ta),
        EditorViewAction::WordEndForward => word_end_forward(ta),
        EditorViewAction::WordEndBack => word_end_back(ta),
        EditorViewAction::LineHead => ta.move_cursor(CursorMove::Head),
        EditorViewAction::LineFirstNonBlank => first_non_blank(ta),
        EditorViewAction::NextLineFirstNonBlank => {
            ta.move_cursor(CursorMove::Down);
            first_non_blank(ta);
        }
        EditorViewAction::PrevLineFirstNonBlank => {
            ta.move_cursor(CursorMove::Up);
            first_non_blank(ta);
        }
        EditorViewAction::ReplaceChar => {
            state.pending = Pending::Replace;
        }
        EditorViewAction::FindCharForward => {
            state.pending = Pending::Find(CharSearch {
                forward: true,
                till: false,
            });
        }
        EditorViewAction::FindCharBackward => {
            state.pending = Pending::Find(CharSearch {
                forward: false,
                till: false,
            });
        }
        EditorViewAction::TillCharForward => {
            state.pending = Pending::Find(CharSearch {
                forward: true,
                till: true,
            });
        }
        EditorViewAction::TillCharBackward => {
            state.pending = Pending::Find(CharSearch {
                forward: false,
                till: true,
            });
        }
        EditorViewAction::RepeatFind => {
            if let Some((search, target)) = state.last_find {
                char_search(ta, search, target, true);
            }
        }
        EditorViewAction::RepeatFindReverse => {
            if let Some((search, target)) = state.last_find {
                let reversed = CharSearch {
                    forward: !search.forward,
                    till: search.till,
                };
                char_search(ta, reversed, target, true);
            }
        }
        EditorViewAction::JoinLines => {
            out.modified = join_lines(ta);
        }
        EditorViewAction::MatchBrace => match_brace(ta),
        EditorViewAction::LineEnd => ta.move_cursor(CursorMove::End),
        EditorViewAction::FileTop => ta.move_cursor(CursorMove::Top),
        EditorViewAction::FileBottom => ta.move_cursor(CursorMove::Bottom),
        EditorViewAction::ClearSearch => {
            out.effect = SideEffect::ClearSearch;
        }
        EditorViewAction::Yank | EditorViewAction::Delete => {}
    }

    out
}

/// Searches for `target` in the current line and moves the cursor to (or
/// just before, for till-searches) its position. `repeat` skips the adjacent
/// position so `;` on a till-search can advance past the current match.
pub fn char_search(ta: &mut TextArea, search: CharSearch, target: char, repeat: bool) -> bool {
    let (row, col) = ta.cursor();
    let line: Vec<char> = ta.lines()[row].chars().collect();
    if search.forward {
        let mut start = col + 1;
        if search.till && repeat {
            start += 1;
        }
        for (i, &ch) in line.iter().enumerate().skip(start) {
            if ch == target {
                let dest = if search.till { i - 1 } else { i };
                ta.move_cursor(CursorMove::Jump(row as u16, dest as u16));
                return true;
            }
        }
        false
    } else {
        let mut end = col;
        if search.till && repeat {
            end = end.saturating_sub(1);
        }
        for i in (0..end).rev() {
            if line[i] == target {
                let dest = if search.till { i + 1 } else { i };
                ta.move_cursor(CursorMove::Jump(row as u16, dest as u16));
                return true;
            }
        }
        false
    }
}

/// Replaces the character under the cursor without moving it (`r`).
pub fn replace_char(ta: &mut TextArea, ch: char) -> bool {
    let (row, col) = ta.cursor();
    if col < ta.lines()[row].chars().count() {
        ta.delete_next_char();
        ta.insert_char(ch);
        ta.move_cursor(CursorMove::Jump(row as u16, col as u16));
        true
    } else {
        false
    }
}

/// Joins the line below onto the current line with a single space (`J`).
pub fn join_lines(ta: &mut TextArea) -> bool {
    let (row, _) = ta.cursor();
    if row + 1 >= ta.lines().len() {
        return false;
    }
    ta.move_cursor(CursorMove::End);
    let junction_col = ta.cursor().1;
    ta.delete_next_char();
    // Collapse leading whitespace of the joined segment into a single space.
    let line: Vec<char> = ta.lines()[row].chars().collect();
    let mut ws = 0;
    while junction_col + ws < line.len() && line[junction_col + ws].is_whitespace() {
        ws += 1;
    }
    for _ in 0..ws {
        ta.delete_next_char();
    }
    if junction_col > 0 && junction_col < ta.lines()[row].chars().count() {
        ta.insert_char(' ');
        ta.move_cursor(CursorMove::Jump(row as u16, junction_col as u16));
    }
    true
}

/// Deletes `count` whole lines starting at the cursor line (`dd` / `dNd`).
pub fn delete_lines(ta: &mut TextArea, count: usize) {
    for _ in 0..count {
        ta.move_cursor(CursorMove::Head);
        ta.delete_line_by_end();
        if ta.cursor().0 == ta.lines().len() - 1 && ta.lines().len() > 1 {
            ta.delete_char();
        } else {
            ta.delete_next_char();
        }
    }
}

/// Yanks `count` whole lines starting at the cursor line (`yy` / `yNy`).
pub fn yank_lines(ta: &mut TextArea, count: usize) {
    let orig = ta.cursor();
    ta.move_cursor(CursorMove::Head);
    ta.start_selection();
    for _ in 0..count {
        ta.move_cursor(CursorMove::Down);
    }
    if ta.cursor().0 < orig.0 + count {
        ta.move_cursor(CursorMove::End);
    }
    ta.copy();
    ta.cancel_selection();
    ta.move_cursor(CursorMove::Jump(orig.0 as u16, orig.1 as u16));
}

/// Pastes yanked text after the cursor (`p`), line-wise if the yank ends with a newline.
pub fn paste_after(ta: &mut TextArea) {
    let text = ta.yank_text();
    let is_line_yank = text.ends_with('\n');
    if is_line_yank {
        let old_row = ta.cursor().0;
        ta.move_cursor(CursorMove::Down);
        if ta.cursor().0 == old_row {
            ta.move_cursor(CursorMove::End);
            ta.insert_newline();
            ta.paste();
            ta.delete_char();
        } else {
            ta.move_cursor(CursorMove::Head);
            ta.paste();
        }
    } else {
        ta.move_cursor(CursorMove::Forward);
        ta.paste();
        ta.move_cursor(CursorMove::Back);
    }
}

/// Pastes yanked text before the cursor (`P`), line-wise if the yank ends with a newline.
pub fn paste_before(ta: &mut TextArea) {
    let text = ta.yank_text();
    let is_line_yank = text.ends_with('\n');
    if is_line_yank {
        ta.move_cursor(CursorMove::Head);
        ta.paste();
    } else {
        ta.paste();
    }
}

/// Moves to the start of the next whitespace-separated word (`W`).
pub fn word_forward_whitespace(ta: &mut TextArea) {
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
        ta.move_cursor(CursorMove::Jump(r as u16, c as u16));
    }
}

/// Moves to the start of the previous whitespace-separated word (`B`).
pub fn word_back_whitespace(ta: &mut TextArea) {
    let lines = ta.lines();
    let (mut r, mut c) = ta.cursor();
    if r < lines.len() {
        if c > 0 {
            c -= 1;
        } else if r > 0 {
            r -= 1;
            c = lines[r].chars().count();
            c = c.saturating_sub(1);
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
                c = c.saturating_sub(1);
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
        ta.move_cursor(CursorMove::Jump(r as u16, c as u16));
    }
}

/// Moves to the end of the current/next word (`e`).
pub fn word_end_forward(ta: &mut TextArea) {
    let lines = ta.lines();
    let (mut r, mut c) = ta.cursor();
    if r < lines.len() {
        let mut current_line: Vec<char> = lines[r].chars().collect();
        if c < current_line.len() {
            c += 1;
        }
        let mut in_word = c < current_line.len() && !current_line[c].is_whitespace();

        loop {
            if c >= current_line.len() {
                if in_word {
                    c = c.saturating_sub(1);
                    break;
                }
                r += 1;
                c = 0;
                if r >= lines.len() {
                    r = lines.len() - 1;
                    c = lines[r].chars().count();
                    c = c.saturating_sub(1);
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
        ta.move_cursor(CursorMove::Jump(r as u16, c as u16));
    }
}

/// Moves to the end of the previous word (`ge`).
pub fn word_end_back(ta: &mut TextArea) {
    let lines = ta.lines();
    let (mut r, mut c) = ta.cursor();
    if r < lines.len() {
        if c > 0 {
            c -= 1;
        } else if r > 0 {
            r -= 1;
            c = lines[r].chars().count();
            c = c.saturating_sub(1);
        }

        let mut current_line: Vec<char> = lines[r].chars().collect();
        let mut skipping_current = c < current_line.len() && !current_line[c].is_whitespace();

        loop {
            if c == 0 && (current_line.is_empty() || r == 0) {
                if r == 0 {
                    c = 0;
                    break;
                }
                r -= 1;
                current_line = lines[r].chars().collect();
                c = current_line.len();
                c = c.saturating_sub(1);
            } else {
                let is_ws = current_line.get(c).is_none_or(|ch| ch.is_whitespace());
                if skipping_current {
                    if is_ws {
                        skipping_current = false;
                    }
                } else if !is_ws {
                    break;
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
                    c = c.saturating_sub(1);
                }
            }
        }
        ta.move_cursor(CursorMove::Jump(r as u16, c as u16));
    }
}

/// Moves to the first non-blank character of the current line (`^`).
pub fn first_non_blank(ta: &mut TextArea) {
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
        ta.move_cursor(CursorMove::Jump(r as u16, new_col as u16));
    }
}

/// Jumps to the matching brace, bracket, or parenthesis (`%`).
pub fn match_brace(ta: &mut TextArea) {
    let lines = ta.lines();
    let (r, c) = ta.cursor();
    if r < lines.len() {
        let current_line: Vec<char> = lines[r].chars().collect();
        if c < current_line.len() {
            let ch = current_line[c];
            let (open, close, dir) = match ch {
                '(' => ('(', ')', 1),
                '{' => ('{', '}', 1),
                '[' => ('[', ']', 1),
                ')' => ('(', ')', -1),
                '}' => ('{', '}', -1),
                ']' => ('[', ']', -1),
                _ => ('\0', '\0', 0),
            };
            if dir != 0 {
                let mut nest = 0;
                let mut tr = r as isize;
                let mut tc = c as isize;
                let mut found = false;
                'outer: while tr >= 0 && (tr as usize) < lines.len() {
                    let t_line: Vec<char> = lines[tr as usize].chars().collect();
                    while tc >= 0 && (tc as usize) < t_line.len() {
                        let t_ch = t_line[tc as usize];
                        if t_ch == open {
                            nest += dir;
                        } else if t_ch == close {
                            nest -= dir;
                        }
                        if nest == 0 {
                            found = true;
                            break 'outer;
                        }
                        tc += dir;
                    }
                    tr += dir;
                    if tr >= 0 && (tr as usize) < lines.len() {
                        let next_len = lines[tr as usize].chars().count();
                        tc = if dir == 1 { 0 } else { (next_len as isize) - 1 };
                    }
                }
                if found {
                    ta.move_cursor(CursorMove::Jump(tr as u16, tc as u16));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn ta(content: &str) -> TextArea<'static> {
        TextArea::new(content.lines().map(ToString::to_string).collect())
    }

    fn press(state: &mut VimState, textarea: &mut TextArea, c: char) -> ViewOutcome {
        handle_view_key(
            state,
            textarea,
            &KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
        )
    }

    #[test]
    fn test_dd_deletes_line() {
        let mut state = VimState::default();
        let mut t = ta("line one\nline two\nline three");
        press(&mut state, &mut t, 'd');
        assert!(matches!(state.pending, Pending::Op { .. }));
        let out = press(&mut state, &mut t, 'd');
        assert!(out.modified);
        assert_eq!(t.lines(), ["line two", "line three"]);
        assert_eq!(state.pending, Pending::None);
    }

    #[test]
    fn test_d2d_deletes_two_lines() {
        let mut state = VimState::default();
        let mut t = ta("one\ntwo\nthree");
        press(&mut state, &mut t, 'd');
        press(&mut state, &mut t, '2');
        press(&mut state, &mut t, 'd');
        assert_eq!(t.lines(), ["three"]);
    }

    #[test]
    fn test_yy_then_p_duplicates_line() {
        let mut state = VimState::default();
        let mut t = ta("alpha\nbeta");
        press(&mut state, &mut t, 'y');
        press(&mut state, &mut t, 'y');
        press(&mut state, &mut t, 'p');
        assert_eq!(t.lines(), ["alpha", "alpha", "beta"]);
    }

    #[test]
    fn test_gg_moves_to_top() {
        let mut state = VimState::default();
        let mut t = ta("one\ntwo\nthree");
        t.move_cursor(CursorMove::Bottom);
        assert_eq!(t.cursor().0, 2);
        press(&mut state, &mut t, 'g');
        press(&mut state, &mut t, 'g');
        assert_eq!(t.cursor(), (0, 0));
    }

    #[test]
    fn test_r_replaces_char_under_cursor() {
        let mut state = VimState::default();
        let mut t = ta("hallo");
        t.move_cursor(CursorMove::Jump(0, 1));
        press(&mut state, &mut t, 'r');
        assert_eq!(state.pending, Pending::Replace);
        let out = press(&mut state, &mut t, 'e');
        assert!(out.modified);
        assert_eq!(t.lines(), ["hello"]);
        assert_eq!(t.cursor(), (0, 1));
    }

    #[test]
    fn test_r_on_empty_line_does_nothing() {
        let mut state = VimState::default();
        let mut t = ta("");
        press(&mut state, &mut t, 'r');
        let out = press(&mut state, &mut t, 'x');
        assert!(!out.modified);
        assert_eq!(t.lines(), [""]);
    }

    #[test]
    fn test_j_joins_lines_with_single_space() {
        let mut state = VimState::default();
        let mut t = ta("foo\n   bar\nbaz");
        let out = press(&mut state, &mut t, 'J');
        assert!(out.modified);
        assert_eq!(t.lines(), ["foo bar", "baz"]);
        assert_eq!(t.cursor(), (0, 3));
    }

    #[test]
    fn test_j_on_last_line_does_nothing() {
        let mut state = VimState::default();
        let mut t = ta("only");
        let out = press(&mut state, &mut t, 'J');
        assert!(!out.modified);
        assert_eq!(t.lines(), ["only"]);
    }

    #[test]
    fn test_f_moves_to_char() {
        let mut state = VimState::default();
        let mut t = ta("one two three");
        press(&mut state, &mut t, 'f');
        press(&mut state, &mut t, 't');
        assert_eq!(t.cursor(), (0, 4));
        // `;` repeats the search
        press(&mut state, &mut t, ';');
        assert_eq!(t.cursor(), (0, 8));
        // `,` reverses it
        press(&mut state, &mut t, ',');
        assert_eq!(t.cursor(), (0, 4));
    }

    #[test]
    fn test_t_stops_before_char() {
        let mut state = VimState::default();
        let mut t = ta("one two three");
        press(&mut state, &mut t, 't');
        press(&mut state, &mut t, 'w');
        assert_eq!(t.cursor(), (0, 4)); // stops on 't', just before 'w' at 5
        // repeat skips past the adjacent match position
        press(&mut state, &mut t, ';');
        assert_eq!(t.cursor(), (0, 4)); // only one 'w'; cursor stays
    }

    #[test]
    fn test_big_f_searches_backward() {
        let mut state = VimState::default();
        let mut t = ta("one two three");
        t.move_cursor(CursorMove::End);
        press(&mut state, &mut t, 'F');
        press(&mut state, &mut t, 'e');
        assert_eq!(t.cursor(), (0, 12));
        press(&mut state, &mut t, ';');
        assert_eq!(t.cursor(), (0, 11));
    }

    #[test]
    fn test_f_no_match_stays_put() {
        let mut state = VimState::default();
        let mut t = ta("abc");
        press(&mut state, &mut t, 'f');
        press(&mut state, &mut t, 'z');
        assert_eq!(t.cursor(), (0, 0));
    }

    #[test]
    fn test_insert_side_effects() {
        let mut state = VimState::default();
        let mut t = ta("hello");
        let out = press(&mut state, &mut t, 'i');
        assert_eq!(out.effect, SideEffect::EnterInsert);
        let out = press(&mut state, &mut t, 'A');
        assert_eq!(out.effect, SideEffect::EnterInsert);
        assert_eq!(t.cursor(), (0, 5));
    }
}
