# Contributing to Ferronote

Thank you for your interest in contributing to Ferronote! 🎉

## Getting Started

1. **Fork** the repository
2. **Clone** your fork: `git clone https://github.com/YOUR_USERNAME/ferronote.git`
3. **Build**: `cargo build`
4. **Test**: `cargo test`
5. **Create a branch**: `git checkout -b feat/your-feature`

## Development Setup

### Prerequisites
- Rust (latest stable, edition 2024)
- A terminal emulator that supports 256 colors

### Running
```bash
cargo run -- --dir ./test-notes
```

### Quality Checks
Before submitting a PR, ensure:
```bash
cargo fmt --check        # Code formatting
cargo clippy -- -W clippy::pedantic  # Linting
cargo test               # All tests pass
```

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — New feature
- `fix:` — Bug fix
- `refactor:` — Code change that neither fixes a bug nor adds a feature
- `docs:` — Documentation only
- `test:` — Adding or updating tests
- `chore:` — Tooling, CI, dependencies

## Architecture

Read [`.gemini/AGENTS.md`](.gemini/AGENTS.md) for a full architecture overview.

The key pattern: **Event → Action → Update State → Render** (The Elm Architecture).

## Pull Request Process

1. Update documentation if you're changing behavior
2. Add tests for new functionality
3. Ensure CI passes
4. Request review from a maintainer
