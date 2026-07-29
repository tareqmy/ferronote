# 2. Adopt The Elm Architecture (TEA) for TUI State Management

Date: 2026-07-29

## Status

Accepted

## Context

Terminal UI applications require predictable state management, event routing, and rendering loops. Unstructured state mutation often leads to rendering inconsistencies and race conditions.

## Decision

We adopt **The Elm Architecture (TEA)** pattern in Rust:
- **Model**: Centralized application state (`App` in `src/app.rs`).
- **Action**: Enum representing explicit state transitions (`src/action.rs`).
- **Update**: Pure function transitions `(State, Action) -> State`.
- **View**: Declarative Ratatui widget rendering loop.

## Consequences

- All state transitions pass through `App::update(Action)`.
- Centralized event handling simplifies testing and debugging.
- UI rendering remains decoupled from background I/O operations.
