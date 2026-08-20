//! Bridges yanked text to the operating system clipboard so text copied in
//! the editor can be pasted into other applications, and text copied in other
//! applications can be pasted with `p` / `P`.

#[cfg(not(test))]
mod imp {
    use std::sync::{Mutex, OnceLock};

    struct State {
        // Kept alive for the lifetime of the process: on X11 the clipboard
        // contents are owned by this handle and would vanish if dropped.
        clipboard: Option<arboard::Clipboard>,
        // What we last wrote to (or read from) the system clipboard. When the
        // clipboard still matches this, nothing external was copied and paste
        // sticks with the internal yank buffer — keeping `dd` + `p` working,
        // since deletes only fill the internal buffer.
        last_synced: Option<String>,
    }

    static STATE: OnceLock<Mutex<State>> = OnceLock::new();

    fn state() -> &'static Mutex<State> {
        STATE.get_or_init(|| {
            Mutex::new(State {
                clipboard: arboard::Clipboard::new().ok(),
                last_synced: None,
            })
        })
    }

    /// Copies `text` to the system clipboard. Failures (e.g. no display
    /// server) are ignored; the in-app yank buffer still works without it.
    pub fn copy_to_system(text: &str) {
        let Ok(mut guard) = state().lock() else {
            return;
        };
        let s = &mut *guard;
        if let Some(clipboard) = s.clipboard.as_mut()
            && clipboard.set_text(text.to_string()).is_ok()
        {
            s.last_synced = Some(text.to_string());
        }
    }

    /// Returns the system clipboard text if it was set by another application
    /// since our last sync, or `None` when pasting should use the internal
    /// yank buffer.
    pub fn external_text() -> Option<String> {
        let mut guard = state().lock().ok()?;
        let s = &mut *guard;
        let text = s.clipboard.as_mut()?.get_text().ok()?;
        if text.is_empty() || s.last_synced.as_deref() == Some(text.as_str()) {
            return None;
        }
        s.last_synced = Some(text.clone());
        Some(text)
    }
}

pub use imp::{copy_to_system, external_text};

// Unit tests exercise yank and paste commands heavily; don't clobber or read
// the developer's real clipboard while they run.
#[cfg(test)]
mod imp {
    pub fn copy_to_system(_text: &str) {}

    pub fn external_text() -> Option<String> {
        None
    }
}
