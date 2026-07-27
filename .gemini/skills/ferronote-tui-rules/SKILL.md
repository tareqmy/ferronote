---
name: Ferronote TUI Guidelines
description: Use this skill when modifying terminal rendering logic, UI components, or error handling in the Ferronote project.
---

# Ferronote TUI & Safety Guidelines

Ferronote is a TUI app built with `ratatui` and `crossterm`. Performance, reliability, and safety are paramount.

## Safety & Error Handling

1. **Terminal Restoration**: ANY code path that exits the application (success, expected error, or panic) MUST restore the terminal from raw mode and leave the alternate screen. Never leave the user's terminal in a broken state.
2. **No Unwraps**: Never use `.unwrap()` in production code. Use `.expect("reason")` only if it's truly an invariant. Otherwise, propagate errors using `?` and `color_eyre::Result`.
3. **Structured Logging**: Use `tracing` macros (`info!`, `debug!`, `warn!`) instead of `println!` or `eprintln!`.

## Rendering Performance

1. **16ms Budget**: Keep rendering under 16ms (60 fps target).
2. **No I/O in Render**: Never perform disk I/O, network calls, or blocking operations inside `draw()` or rendering functions.
3. **Data Safety**: Always trigger an auto-save after inactivity. No more than 1 second of typing should be lost on a crash. Destructive actions like deleting a note must prompt for explicit confirmation.

When building new components, implement the `Widget` trait and rely purely on the passed-in application state.
