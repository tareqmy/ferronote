use crate::search::SearchResult;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

#[derive(Debug, Default)]
pub struct NoteList {
    pub state: ListState,
    pub items: Vec<SearchResult>,
}

impl NoteList {
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

    #[must_use]
    pub fn selected_note(&self) -> Option<String> {
        self.state
            .selected()
            .and_then(|i| self.items.get(i).map(|res| res.filename.clone()))
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, is_focused: bool) {
        let border_color = if is_focused {
            Color::Blue
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Notes ");

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| {
                if i.is_create_prompt {
                    let text = format!(" + {}", i.title);
                    return ListItem::new(Line::from(Span::styled(
                        text,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::ITALIC),
                    )));
                }

                let mut spans = Vec::new();
                for (idx, ch) in i.title.chars().enumerate() {
                    let mut style = Style::default();
                    if i.title_match_indices.contains(&idx) {
                        style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                    }
                    spans.push(Span::styled(ch.to_string(), style));
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
                .bg(Color::Blue) // #458588
                .fg(Color::White)
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
