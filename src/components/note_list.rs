use ferronote_store::format_timestamp;
use ferronote_search::SearchResult;
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
    /// Event queue for emitting app actions.
    pub queue: crate::queue::Queue,
}

impl NoteList {
    /// Creates a new empty `NoteList` component.
    #[must_use]
    pub fn new(queue: crate::queue::Queue) -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
            queue,
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
                    self.items.len() - 1
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
                    0
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
        sort_order: &str,
        theme: &crate::theme::ThemePalette,
    ) {
        let border_color = if is_focused {
            theme.border_active
        } else {
            theme.border_inactive
        };

        let sort_title = match sort_order {
            "modified_desc" => "Sort: Modified ↓",
            "modified_asc" => "Sort: Modified ↑",
            "title_asc" => "Sort: Title A-Z",
            "title_desc" => "Sort: Title Z-A",
            "created_desc" => "Sort: Created ↓",
            "created_asc" => "Sort: Created ↑",
            _ => sort_order,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Notes ")
            .title_top(ratatui::text::Line::from(format!(" {} ", sort_title)).alignment(ratatui::layout::Alignment::Right));

        let available_width = (area.width as usize).saturating_sub(4);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| {
                if i.is_create_prompt {
                    let text = format!(" + {}", i.title);
                    let text_chars: Vec<char> = text.chars().collect();
                    let display_text = if text_chars.len() > available_width {
                        if available_width >= 3 {
                            let prefix: String = text_chars[..available_width - 3].iter().collect();
                            format!("{prefix}...")
                        } else {
                            ".".repeat(available_width)
                        }
                    } else {
                        text
                    };
                    return ListItem::new(Line::from(Span::styled(
                        display_text,
                        Style::default()
                            .fg(theme.search_match)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }

                let date_str = if show_modified_time {
                    format_timestamp(i.modified_at)
                } else {
                    String::new()
                };

                let date_len = date_str.chars().count();
                let show_date = !date_str.is_empty() && available_width > date_len + 1;

                let pin_prefix_len = if i.is_pinned { 2 } else { 0 };

                let max_title_len = if show_date {
                    available_width.saturating_sub(date_len + 1 + pin_prefix_len)
                } else {
                    available_width.saturating_sub(pin_prefix_len)
                };

                let title_chars: Vec<char> = i.title.chars().collect();
                let is_title_curtailed = title_chars.len() > max_title_len;

                let mut spans = Vec::new();

                if i.is_pinned {
                    spans.push(Span::styled("📌 ", Style::default().fg(theme.accent)));
                }

                if is_title_curtailed {
                    if max_title_len >= 3 {
                        let visible_len = max_title_len - 3;
                        for idx in 0..visible_len {
                            let ch = title_chars[idx];
                            let mut style = Style::default();
                            if i.title_match_indices.contains(&idx) {
                                style = style.fg(theme.search_match).add_modifier(Modifier::BOLD);
                            }
                            spans.push(Span::styled(ch.to_string(), style));
                        }
                        spans.push(Span::raw("..."));
                    } else {
                        for _ in 0..max_title_len {
                            spans.push(Span::raw("."));
                        }
                    }
                } else {
                    for (idx, ch) in title_chars.iter().enumerate() {
                        let mut style = Style::default();
                        if i.title_match_indices.contains(&idx) {
                            style = style.fg(theme.search_match).add_modifier(Modifier::BOLD);
                        }
                        spans.push(Span::styled(ch.to_string(), style));
                    }
                }

                if show_date {
                    let rendered_title_len = if is_title_curtailed {
                        max_title_len
                    } else {
                        title_chars.len()
                    };

                    let padding = available_width.saturating_sub(rendered_title_len + date_len);

                    if padding > 0 {
                        spans.push(Span::raw(" ".repeat(padding)));
                    }
                    spans.push(Span::styled(date_str, Style::default().fg(Color::DarkGray)));
                }

                let mut lines = vec![Line::from(spans)];

                if let Some(preview) = &i.content_preview {
                    let preview_chars: Vec<char> = preview.chars().collect();
                    let display_preview = if preview_chars.len() > available_width {
                        if available_width >= 3 {
                            let prefix: String = preview_chars[..available_width - 3].iter().collect();
                            format!("{prefix}...")
                        } else {
                            ".".repeat(available_width)
                        }
                    } else {
                        preview.clone()
                    };
                    lines.push(Line::from(Span::styled(
                        display_preview,
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

impl crate::components::Component for NoteList {
    fn event(&mut self, ev: &crate::event::Event) -> color_eyre::Result<crate::components::EventState> {
        if let crate::event::Event::Key(key) = ev {
            use ratatui::crossterm::event::KeyCode;
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next();
                    self.queue.push(crate::action::Action::SelectNote(self.selected_note()));
                    return Ok(crate::components::EventState::Consumed);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous();
                    self.queue.push(crate::action::Action::SelectNote(self.selected_note()));
                    return Ok(crate::components::EventState::Consumed);
                }
                KeyCode::PageDown => {
                    self.page_down(10);
                    self.queue.push(crate::action::Action::SelectNote(self.selected_note()));
                    return Ok(crate::components::EventState::Consumed);
                }
                KeyCode::PageUp => {
                    self.page_up(10);
                    self.queue.push(crate::action::Action::SelectNote(self.selected_note()));
                    return Ok(crate::components::EventState::Consumed);
                }
                KeyCode::Home => {
                    self.select_first();
                    self.queue.push(crate::action::Action::SelectNote(self.selected_note()));
                    return Ok(crate::components::EventState::Consumed);
                }
                KeyCode::End => {
                    self.select_last();
                    self.queue.push(crate::action::Action::SelectNote(self.selected_note()));
                    return Ok(crate::components::EventState::Consumed);
                }
                _ => {}
            }
        }
        Ok(crate::components::EventState::NotConsumed)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ferronote_search::SearchResult;

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
                is_pinned: false,
            })
            .collect()
    }

    #[test]
    fn test_note_list_navigation() {
        let queue = crate::queue::Queue::new();
        let mut list = NoteList::new(queue);
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
        let queue = crate::queue::Queue::new();
        let mut list = NoteList::new(queue);
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
    fn test_note_list_clamping_and_empty() {
        let queue = crate::queue::Queue::new();
        let mut list = NoteList::new(queue);
        // Empty list navigation
        list.next();
        assert_eq!(list.state.selected(), None);
        list.previous();
        assert_eq!(list.state.selected(), None);
        assert_eq!(list.selected_note(), None);

        // Clamping navigation
        list.set_items(make_mock_results(3));
        assert_eq!(list.state.selected(), Some(0));
        assert_eq!(list.selected_note(), Some("note_0.md".to_string()));

        list.previous(); // clamps to first
        assert_eq!(list.state.selected(), Some(0));
        assert_eq!(list.selected_note(), Some("note_0.md".to_string()));

        list.state.select(Some(2)); // move to last manually
        list.next(); // clamps to last
        assert_eq!(list.state.selected(), Some(2));
        assert_eq!(list.selected_note(), Some("note_2.md".to_string()));

        // Setting fewer items rescales selection
        list.state.select(Some(2));
        list.set_items(make_mock_results(2));
        assert_eq!(list.state.selected(), Some(1));
    }

    #[test]
    fn test_note_list_title_curtailment() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use crate::theme::ThemePalette;

        let queue = crate::queue::Queue::new();
        let mut list = NoteList::new(queue);
        list.set_items(vec![SearchResult {
            filename: "very_long_note_title.md".to_string(),
            title: "This is a very long note title that exceeds available width".to_string(),
            score: 0,
            title_match_indices: Vec::new(),
            content_preview: None,
            is_create_prompt: false,
            modified_at: 0,
            is_pinned: false,
        }]);

        // Box width = 20. Available width = 20 - 4 = 16.
        // Title length (58) > 16, so it must be curtailed to 13 chars + "..." = 16 chars.
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        let theme = ThemePalette::from_name("dark");
        terminal
            .draw(|f| {
                list.draw(
                    f,
                    Rect::new(0, 0, 20, 5),
                    true,
                    false,
                    "modified_desc",
                    &theme,
                );
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let row_content: String = (1..19).map(|x| buffer[(x, 1)].symbol().to_string()).collect();
        assert!(row_content.contains("..."), "Row content should contain '...': {row_content}");

        // Test create prompt curtailment
        let queue2 = crate::queue::Queue::new();
        let mut list_prompt = NoteList::new(queue2);
        list_prompt.set_items(vec![SearchResult {
            filename: "new.md".to_string(),
            title: "Extremely long create prompt note title text".to_string(),
            score: 0,
            title_match_indices: Vec::new(),
            content_preview: None,
            is_create_prompt: true,
            modified_at: 0,
            is_pinned: false,
        }]);

        let backend2 = TestBackend::new(20, 5);
        let mut terminal2 = Terminal::new(backend2).unwrap();
        terminal2
            .draw(|f| {
                list_prompt.draw(
                    f,
                    Rect::new(0, 0, 20, 5),
                    true,
                    false,
                    "modified_desc",
                    &theme,
                );
            })
            .unwrap();

        let buffer2 = terminal2.backend().buffer();
        let prompt_row: String = (1..19).map(|x| buffer2[(x, 1)].symbol().to_string()).collect();
        assert!(prompt_row.contains("..."), "Prompt row should contain '...': {prompt_row}");
    }
}
