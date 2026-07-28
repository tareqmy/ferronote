use ferronote::app::{App, Action};
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind, KeyModifiers};

fn main() {
    // create a temp dir for notes
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(dir.path().to_path_buf());
    
    // type "hello"
    for c in "hello".chars() {
        app.update(Action::HandleKey(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty())));
    }
    
    // Press Enter
    app.update(Action::HandleKey(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }));
    
    println!("Number of notes in index: {}", app.index.search("").len());
    println!("Number of notes in note_list: {}", app.note_list.items.len());
    for item in &app.note_list.items {
        println!("- {} ({:?})", item.title, item.filename);
    }
}
