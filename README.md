<div align="center">
  <img src="assets/logo.jpg" alt="Ferronote Logo" width="220" />
  <h1>🗒️ Ferronote</h1>
  <p><b>Notes at the speed of thought</b> — a blazing-fast terminal note-taking app inspired by <a href="https://notational.net/">Notational Velocity</a>.</p>
</div>

Ferronote is a Rust TUI application that brings the elegance and speed of Notational Velocity to the modern terminal. Search, create, and edit notes without ever touching the mouse.

## ✨ Philosophy

- **Modeless** — The search bar *is* the creation bar. Type to search; if nothing matches, press Enter to create.
- **Keyboard-first** — Every action is a keystroke away. No menus, no buttons, no mouse required.
- **Plain text** — Notes are Markdown files in a folder you choose. No database, no lock-in, no proprietary format.
- **Fast** — Sub-millisecond fuzzy search across thousands of notes. Instant startup.

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/tareqmy/ferronote.git
cd ferronote
cargo build --release

# Run
./target/release/ferronote

# Or specify a notes directory
./target/release/ferronote --dir ~/notes
```

## ⌨️ Keybindings

| Key              | Action                              |
|------------------|-------------------------------------|
| `/` or `Ctrl+L`  | Focus search bar                    |
| `Enter`          | Open selected note / Create new     |
| `↑` / `↓`        | Navigate note list                  |
| `Ctrl+S`         | Save current note                   |
| `Ctrl+D`         | Delete note (with confirmation)     |
| `Ctrl+N`         | New note                            |
| `Esc`            | Back to search / Cancel             |
| `Ctrl+Q`         | Quit                                |

## 🏗️ Architecture

Ferronote uses **The Elm Architecture (TEA)** adapted for Rust:

```
Event → Action → Update State → Render
```

- **Model**: Application state (`app.rs`)
- **Update**: Action processing and state transitions
- **View**: Ratatui widget rendering

Built with:
- [`ratatui`](https://ratatui.rs/) + [`crossterm`](https://docs.rs/crossterm) — Terminal UI
- [`tui-textarea`](https://docs.rs/tui-textarea) — Text editing
- [`fuzzy-matcher`](https://docs.rs/fuzzy-matcher) — Fuzzy search
- [`tokio`](https://tokio.rs/) — Async runtime
- [`clap`](https://docs.rs/clap) — CLI arguments

## 📁 Project Structure

```
src/
├── main.rs              # Entry point, CLI, terminal setup
├── app.rs               # App state & update logic
├── tui.rs               # Terminal backend wrapper
├── event.rs             # Async event polling
├── action.rs            # Action enum (all state transitions)
├── note_store.rs        # Note CRUD & filesystem I/O
├── config.rs            # User configuration
└── components/
    ├── mod.rs           # Component trait & re-exports
    ├── search_bar.rs    # Unified search/create input
    ├── note_list.rs     # Filtered note list
    └── editor.rs        # Markdown editor
```

## 📄 License

MIT
