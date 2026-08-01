use tui_textarea::{TextArea, CursorMove};

fn main() {
    let mut ta = TextArea::new(vec!["Line 1".to_string(), "Line 2".to_string()]);
    ta.set_search_pattern("Line");
    ta.search_forward(false);
}
