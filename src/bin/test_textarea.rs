use tui_textarea::{TextArea, CursorMove};

fn main() {
    let mut ta = TextArea::from(["Hello world!".to_string()]);
    ta.move_cursor(tui_textarea::CursorMove::Jump(0, 5));
    println!("Cursor before: {:?}", ta.cursor());
    println!("Line before: {:?}", ta.lines()[0]);

    let (row, col) = ta.cursor();
    ta.move_cursor(tui_textarea::CursorMove::Head);
    ta.insert_str("    ");
    ta.move_cursor(tui_textarea::CursorMove::Jump(row as u16, (col + 4) as u16));

    println!("Cursor after: {:?}", ta.cursor());
    println!("Line after: {:?}", ta.lines()[0]);
    
    // Test Ctrl+d behavior
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

    println!("Cursor after Ctrl+d: {:?}", ta.cursor());
    println!("Line after Ctrl+d: {:?}", ta.lines()[0]);
}
