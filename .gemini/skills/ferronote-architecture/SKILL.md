---
name: Ferronote Architecture Guidelines
description: Use this skill when modifying the application state, adding new events, or adding new actions in the Ferronote project.
---

# Ferronote Architecture (The Elm Architecture)

Ferronote uses a strict adaptation of **The Elm Architecture (TEA)**. When modifying the application state or adding new features, you must follow this pattern:

## Event → Action → Update State → Render

1. **Events** (`src/event.rs`): Keyboard input, terminal resize, or tick events are captured and mapped to high-level actions.
2. **Actions** (`src/action.rs`): Define all possible state transitions as variants in the `Action` enum (e.g., `CreateNote`, `DeleteNote`, `UpdateSearch`). Name them using `VerbNoun`.
3. **Update State** (`src/app.rs`): The `update()` method processes an `Action` and returns an optional command or new `Action`. **Never mutate state directly from event handlers; always emit an Action.**
4. **Render** (`src/tui.rs` & `src/components/`): Components implement the `Widget` trait and render based on the current state. No I/O or heavy computation should happen during rendering.

## Background Tasks
- Use `tokio::sync::mpsc` channels for background task communication.
- Dispatch heavy operations (like reading large files or saving) to async tasks to avoid blocking the UI thread.
