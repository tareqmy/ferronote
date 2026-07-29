use crate::note_store::format_timestamp;
use crate::search::SearchResult;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

/// Sidebar component rendering a filterable list of note titles with match highlights.
#[derive(Debug, Default)]
pub struct NoteList {
    /// Ratatui state tracking active list selection index.
    pub state: ListState,
    /// Matching search results displayed in sidebar list.
    pub items: Vec<SearchResult>,
}

impl NoteList {
    /// Creates a new empty `NoteList` component.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn set_items(&mut self, items: Vec<SearchResult>) {
        self.items = items;
        // Reset selection if out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.items.len() {
                self.state.select(if self.items.is_empty() {
                    None
                } else {
                    Some(self.items.len() - 1)
                });
            }
        } else if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.state.select(Some(self.items.len() - 1));
        }
    }

    pub fn page_down(&mut self, page_size: usize) {
        if self.items.is_empty() {
            return;
        }
        let current = self.state.selected().unwrap_or(0);
        let target = (current + page_size).min(self.items.len() - 1);
        self.state.select(Some(target));
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.items.is_empty() {
            return;
        }
        let current = self.state.selected().unwrap_or(0);
        let target = current.saturating_sub(page_size);
        self.state.select(Some(target));
    }

    #[must_use]
    pub fn selected_note(&self) -> Option<String> {
        self.state
            .selected()
            .and_then(|i| self.items.get(i).map(|res| res.filename.clone()))
    }

    pub fn click_at(
        &mut self,
        y: u16,
        list_area: Rect,
        _show_modified_time: bool,
    ) -> Option<String> {
        let border_offset = 1;
        if y <= list_area.y || y >= list_area.y + list_area.height.saturating_sub(border_offset) {
            return None;
        }

        let relative_y = (y - list_area.y - border_offset) as usize;
        let scroll_offset = self.state.offset();

        let mut current_line = 0;
        for (i, item) in self.items.iter().enumerate().skip(scroll_offset) {
            let has_sub_text = item.content_preview.is_some();

            let item_lines = if item.is_create_prompt {
                1
            } else if has_sub_text {
                2
            } else {
                1
            };

            if relative_y >= current_line && relative_y < current_line + item_lines {
                self.state.select(Some(i));
                return Some(item.filename.clone());
            }

            current_line += item_lines;
        }

        None
    }

    pub fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        is_focused: bool,
        show_modified_time: bool,
        theme: &crate::theme::ThemePalette,
    ) {
        let border_color = if is_focused {
            theme.border_active
        } else {
            theme.border_inactive
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Notes ");

        let available_width = (area.width as usize).saturating_sub(4);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| {
                if i.is_create_prompt {
                    let text = format!(" + {}", i.title);
                    return ListItem::new(Line::from(Span::styled(
                        text,
                        Style::default()
                            .fg(theme.search_match)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }

                let mut spans = Vec::new();
                for (idx, ch) in i.title.chars().enumerate() {
                    let mut style = Style::default();
                    if i.title_match_indices.contains(&idx) {
                        style = style.fg(theme.search_match).add_modifier(Modifier::BOLD);
                    }
                    spans.push(Span::styled(ch.to_string(), style));
                }

                let date_str = if show_modified_time {
                    format_timestamp(i.modified_at)
                } else {
                    String::new()
                };

                if !date_str.is_empty() {
                    let title_len = i.title.chars().count();
                    let date_len = date_str.chars().count();
                    let padding = available_width.saturating_sub(title_len + date_len);

                    if padding > 0 {
                        spans.push(Span::raw(" ".repeat(padding)));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                    spans.push(Span::styled(date_str, Style::default().fg(Color::DarkGray)));
                }

                let mut lines = vec![Line::from(spans)];

                if let Some(preview) = &i.content_preview {
                    lines.push(Line::from(Span::styled(
                        preview.clone(),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                ListItem::new(lines)
            })
            .collect();

        // Style for selected item
        let highlight_style = if is_focused {
            Style::default()
                .bg(theme.selection_bg)
                .fg(theme.selection_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::SearchResult;

    fn make_mock_results(count: usize) -> Vec<SearchResult> {
        (0..count)
            .map(|i| SearchResult {
                filename: format!("note_{i}.md"),
                title: format!("Note {i}"),
                score: 0,
                title_match_indices: Vec::new(),
                content_preview: None,
                is_create_prompt: false,
                modified_at: 0,
            })
            .collect()
    }

    #[test]
    fn test_note_list_navigation() {
        let mut list = NoteList::new();
        list.set_items(make_mock_results(25));
        assert_eq!(list.state.selected(), Some(0));

        list.page_down(10);
        assert_eq!(list.state.selected(), Some(10));

        list.page_down(10);
        assert_eq!(list.state.selected(), Some(20));

        list.select_last();
        assert_eq!(list.state.selected(), Some(24));

        list.page_up(10);
        assert_eq!(list.state.selected(), Some(14));

        list.select_first();
        assert_eq!(list.state.selected(), Some(0));
    }

    #[test]
    fn test_note_list_click_at() {
        let mut list = NoteList::new();
        list.set_items(make_mock_results(5));
        let list_area = Rect::new(0, 3, 24, 20);

        // Click top border (y = 3) returns None
        assert_eq!(list.click_at(3, list_area, true), None);

        // With show_modified_time = false (1 line per item)
        // Click first item (y = 4)
        assert_eq!(
            list.click_at(4, list_area, false),
            Some("note_0.md".to_string())
        );
        assert_eq!(list.state.selected(), Some(0));

        // Click second item (y = 5)
        assert_eq!(
            list.click_at(5, list_area, false),
            Some("note_1.md".to_string())
        );
        assert_eq!(list.state.selected(), Some(1));

        // Click third item (y = 6)
        assert_eq!(
            list.click_at(6, list_area, false),
            Some("note_2.md".to_string())
        );
        assert_eq!(list.state.selected(), Some(2));
    }

    #[test]
    fn test_note_list_wrapping_and_empty() {
        let mut list = NoteList::new();
        // Empty list navigation
        list.next();
        assert_eq!(list.state.selected(), None);
        list.previous();
        assert_eq!(list.state.selected(), None);
        assert_eq!(list.selected_note(), None);

        // Wrapping navigation
        list.set_items(make_mock_results(3));
        assert_eq!(list.state.selected(), Some(0));
        assert_eq!(list.selected_note(), Some("note_0.md".to_string()));

        list.previous(); // wrap to last
        assert_eq!(list.state.selected(), Some(2));
        assert_eq!(list.selected_note(), Some("note_2.md".to_string()));

        list.next(); // wrap to first
        assert_eq!(list.state.selected(), Some(0));

        // Setting fewer items rescales selection
        list.state.select(Some(2));
        list.set_items(make_mock_results(2));
        assert_eq!(list.state.selected(), Some(1));
    }
}
