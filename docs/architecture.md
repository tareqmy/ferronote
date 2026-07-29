# 🏗️ Architecture & Project Structure

Ferronote is engineered with high performance and modular architecture in mind, using **The Elm Architecture (TEA)** pattern adapted for Rust TUI apps.

---

## 📐 The Elm Architecture (TEA)

```
       +-----------------------+
       |     Event Stream      |
       |  (Keyboard/FS Watcher)|
       +-----------+-----------+
                   |
                   v
          +-----------------+
          |   Action Enum   |
          +--------+--------+
                   |
                   v
+------------------+------------------+
|          Update Function            |
|       (App State Mutation)          |
+------------------+------------------+
                   |
                   v
          +-----------------+
          |    Ratatui      |
          |   View Render   |
          +-----------------+
```

- **Model** (`app.rs`): Holds application state, note collection, search query, and UI active panel.
- **Update** (`action.rs`): Handles decoupled actions and executes deterministic state transitions.
- **View** (`components/`): Renders terminal UI widgets cleanly using `ratatui`.

---

## 🛠️ Core Dependencies & Tech Stack

- [`ratatui`](https://ratatui.rs/) + [`crossterm`](https://docs.rs/crossterm) — Terminal UI rendering and cross-platform terminal control.
- [`tui-textarea`](https://docs.rs/tui-textarea) — Multi-line Markdown editor component.
- [`fuzzy-matcher`](https://docs.rs/fuzzy-matcher) — Fast Skim fuzzy match scoring engine.
- [`tokio`](https://tokio.rs/) — Asynchronous runtime and real-time filesystem event monitoring.
- [`clap`](https://docs.rs/clap) — Declarative CLI parameter parsing.
- [`serde`](https://serde.rs/) — Configuration and metadata serialization.

---

## 📁 Source Directory Structure

```
src/
├── main.rs              # Entry point, CLI flags, terminal setup
├── lib.rs               # Library module exports
├── app.rs               # App state & Elm update loop logic
├── tui.rs               # Terminal backend initialization & restoration
├── event.rs             # Async crossterm event & file system watcher
├── action.rs            # Action enum representing state transitions
├── focus.rs             # Focus management enum (SearchBar, NoteList, Editor)
├── search.rs            # Skim fuzzy search index, tag parser & backlinks
├── theme.rs             # Theme palette manager (default, gruvbox, nord, dracula)
├── note_store.rs        # Note CRUD, trash handling, atomic disk I/O, import/export
├── config.rs            # User settings loading, saving & default fallbacks
└── components/
    ├── mod.rs           # Component trait & re-exports
    ├── search_bar.rs    # Unified search/create input bar
    ├── note_list.rs     # Filtered note title list with match highlights
    └── editor.rs        # Markdown note body editor with wiki-link extraction
```
