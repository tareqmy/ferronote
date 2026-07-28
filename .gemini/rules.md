# Gemini Rules for Ferronote

## Context
- This is a Rust TUI application built with ratatui + crossterm
- It follows The Elm Architecture (TEA): Event → Action → Update State → Render
- Notes are stored as plain `.md` files on disk
- Read `.gemini/AGENTS.md` for full architecture and conventions
- Read `.gemini/STYLE_GUIDE.md` for UI layout and interaction patterns

## Critical Rules
1. **Always restore the terminal** — any code path that exits (success, error, panic) must
   restore the terminal from raw mode and leave the alternate screen.
2. **Never `.unwrap()` in production code** — use `?` with `color_eyre::Result`.
3. **Keep rendering under 16ms** — never do I/O inside `draw()` functions.
4. **Auto-save** — the editor must auto-save after inactivity. Never lose user data.
5. **Run `cargo fmt` and `cargo clippy` before committing.**
6. **Use Conventional Commits** for all commit messages.
7. **Modularity** — code must be separated into relevant modules/structs (Single Responsibility Principle).
8. **Test Coverage** — code must be covered with test cases.
9. **Synchronize Default Welcome Note** — whenever documentation or keybindings are updated, ensure the default welcome note (`create_default_welcome_note` in `src/note_store.rs`) is also updated.


## Preferred Patterns
- Use `Action` enum for all state transitions (never mutate state directly from event handlers)
- Use `tokio::sync::mpsc` channels for background task communication
- Implement the `Widget` trait for custom UI components
- Use `tracing` macros (`info!`, `debug!`, `warn!`) instead of `println!` or `eprintln!`
