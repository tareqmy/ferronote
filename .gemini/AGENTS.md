# Ferronote — AI Agent Instructions

## Project Overview

**Ferronote** is a Rust TUI (Terminal User Interface) application that serves as a modern,
cross-platform replacement for [Notational Velocity](https://notational.net/). It is designed
around the same core philosophy: **notes at the speed of thought** — a modeless, keyboard-driven,
search-first interface where finding and creating notes are unified into one fluid action.

## Core Design Philosophy

1. **Modeless Operation**: There is no distinction between "searching" and "creating". The single
   input field is always active. If a search yields results, navigate them. If no match exists,
   pressing Enter creates a new note with that title.
2. **Keyboard-First**: The entire UI is designed for keyboard-only operation. No mouse required.
   Every action has a keyboard shortcut.
3. **Speed Over Features**: Sub-millisecond search, instant note creation, zero startup delay.
   Performance is a feature.
4. **Plain Text Files**: Notes are stored as individual `.md` (Markdown) files in a user-configured
   directory. No proprietary database. No lock-in.
5. **Minimal & Focused**: Resist feature creep. Each feature must justify its existence against the
   cost of added complexity.

## Technology Stack

| Layer          | Technology                      | Purpose                           |
|----------------|---------------------------------|-----------------------------------|
| Language       | Rust (edition 2024)             | Performance, safety, correctness  |
| TUI Framework  | `ratatui` + `crossterm`         | Terminal rendering & input        |
| Async Runtime  | `tokio`                         | Background I/O, file watching     |
| Text Editing   | `tui-textarea`                  | In-terminal text editing widget   |
| Search         | `fuzzy-matcher` (skim algorithm)| Fuzzy note title/content search   |
| Serialization  | `serde` + `serde_json`          | Config & metadata persistence     |
| CLI            | `clap` (derive)                 | Command-line argument parsing     |
| Error Handling | `color-eyre`                    | Rich, colored error reports       |
| Logging        | `tracing` + `tracing-subscriber`| Structured diagnostic logging     |
| Timestamps     | `chrono`                        | Note creation/modification times  |
| User Dirs      | `dirs`                          | Cross-platform path resolution    |

## Architecture

The application follows **The Elm Architecture (TEA)** adapted for Rust:

```
┌─────────────────────────────────────────────────┐
│                   main.rs                       │
│  - Entry point                                  │
│  - CLI parsing (clap)                           │
│  - Terminal init/restore (tui.rs)               │
│  - Launches the main event loop                 │
└──────────────────────┬──────────────────────────┘
                       │
          ┌────────────▼────────────┐
          │        app.rs           │
          │  - App state (Model)    │
          │  - update() handles     │
          │    Action → State       │
          │  - Owns NoteStore       │
          └────────────┬────────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   ┌─────────┐  ┌───────────┐  ┌──────────┐
   │ action.rs│  │ event.rs  │  │  tui.rs  │
   │ (Action  │  │ (Event    │  │ (Terminal │
   │  enum)   │  │  polling) │  │  setup)  │
   └─────────┘  └───────────┘  └──────────┘
        │
        ▼
   ┌──────────────────────┐
   │   components/        │
   │   ├── search_bar.rs  │  ← Unified search/create input
   │   ├── note_list.rs   │  ← Filtered list of matching notes
   │   └── editor.rs      │  ← Note body editor (tui-textarea)
   └──────────────────────┘
        │
        ▼
   ┌──────────────────────┐
   │   note_store.rs      │  ← File I/O, indexing, search
   └──────────────────────┘
```

### Source File Responsibilities

| File                       | Responsibility                                           |
|----------------------------|----------------------------------------------------------|
| `src/main.rs`              | Entry point: CLI parsing, terminal init, run loop        |
| `src/lib.rs`               | Library exports and module declarations                  |
| `src/app.rs`               | Application state, update logic, mode management         |
| `src/tui.rs`               | Terminal backend setup, enter/exit alternate screen       |
| `src/event.rs`             | Async event polling (keyboard, resize, tick, watcher)    |
| `src/action.rs`            | Centralized `Action` enum for all state transitions      |
| `src/focus.rs`             | Focus state enum (SearchBar, NoteList, Editor)           |
| `src/search.rs`            | Skim fuzzy search index, `#tag` parser, backlinks        |
| `src/theme.rs`             | Theme palette manager (default, gruvbox, nord, dracula)  |
| `src/note_store.rs`        | Note CRUD, filesystem I/O, trash, import/export          |
| `src/config.rs`            | User configuration loading, default fallbacks, saving    |
| `src/components/mod.rs`    | Component trait definition, re-exports                   |
| `src/components/search_bar.rs` | Search/create input widget                           |
| `src/components/note_list.rs`  | Scrollable, filterable note list with highlights     |
| `src/components/editor.rs`     | Markdown note editor using tui-textarea & wiki links |

## Coding Conventions

### Rust Style
- Follow **`rustfmt`** defaults. Run `cargo fmt` before every commit.
- Follow **`clippy`** recommendations. Run `cargo clippy -- -W clippy::pedantic` and fix warnings.
- Use `color_eyre::Result` as the default Result type.
- Prefer `thiserror` for library-style errors, `color-eyre` for application-level errors.
- Use `tracing::instrument` on non-trivial functions for structured logging.

### Naming
- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Action variants: `VerbNoun` (e.g., `CreateNote`, `DeleteNote`, `UpdateSearch`)

### Error Handling
- Never use `.unwrap()` in production code. Use `.expect("reason")` only for truly invariant conditions.
- Propagate errors with `?`. Add context with `.wrap_err("context")`.
- Panic hooks must restore the terminal before crashing.

### Testing
- Unit tests go in the same file under `#[cfg(test)] mod tests { ... }`.
- Integration tests go in `tests/`.
- Aim for test coverage on all `NoteStore` operations and `Action` → state transitions.

### Git Conventions
- **Commit messages**: Follow [Conventional Commits](https://www.conventionalcommits.org/).
  - `feat:` for new features
  - `fix:` for bug fixes
  - `refactor:` for non-functional changes
  - `docs:` for documentation
  - `test:` for test additions/changes
  - `chore:` for tooling, CI, dependencies
- **Branch naming**: `feat/short-description`, `fix/short-description`

## Key Behaviors to Preserve

When modifying the codebase, **always** ensure these invariants hold:

1. **Terminal restoration**: The terminal must be restored to its original state on exit,
   panic, or error. Never leave the user in raw mode.
2. **File safety**: Notes must never be silently overwritten or deleted. Any destructive
   operation requires explicit user confirmation.
3. **Search responsiveness**: Search/filter must remain under 16ms for up to 10,000 notes
   to maintain 60fps rendering.
4. **No data loss**: Auto-save on every edit. If the app crashes, no more than 1 second
   of typing should be lost.
5. **Startup speed**: Cold start must be under 100ms for up to 10,000 notes.

## Dependencies Policy

- Prefer well-maintained, widely-used crates from the Rust ecosystem.
- Pin major versions in `Cargo.toml` (e.g., `ratatui = "0.29"`).
- Audit new dependencies for: maintenance status, license compatibility (MIT/Apache-2.0),
  binary size impact, and compile-time impact.
- Avoid `unsafe` unless absolutely necessary and clearly documented.

## File Format

Notes are stored as plain Markdown (`.md`) files:
- **Filename** = note title (sanitized for filesystem safety) + `.md`
- **Content** = raw Markdown body
- **Metadata** (optional): stored in a sidecar `.ferronote/metadata.json` in the notes directory,
  keyed by filename, containing `created_at`, `modified_at`, and `tags`.
