<p align="center">
  <img src="assets/logo.jpg" alt="Ferronote Logo" width="650" />
</p>

<p align="center">
  <b>Notes at the speed of thought</b> — a blazing-fast terminal note-taking app inspired by <a href="https://notational.net/">Notational Velocity</a>.
</p>

Ferronote is a Rust TUI application that brings the elegance and speed of Notational Velocity to the modern terminal. Search, create, and edit notes without ever touching the mouse.

## ✨ Philosophy & Key Features

- **Modeless** — The search bar *is* the creation bar. Type to search; if nothing matches, press Enter to create.
- **Keyboard-first** — Every action is a keystroke away. No menus, no buttons, no mouse required.
- **Plain text** — Notes are Markdown files in a folder you choose. No database, no lock-in, no proprietary format.
- **Fast** — Sub-millisecond fuzzy search across thousands of notes. Instant startup.
- **🏷️ Tag Filtering** — Organize with `#tag` annotations and instantly filter notes by `#tag` search query.
- **🔗 Wiki Links & Backlinks** — Link notes seamlessly using `[[Note Title]]` syntax with automatic backlink detection.
- **🎨 Custom Themes** — Personalize your terminal experience with pre-built themes (`default`, `gruvbox`, `nord`, `dracula`).
- **⚙️ Interactive Settings** — Tweak options dynamically in the interactive Settings Overlay (`Ctrl+P`).
- **🗑️ Trash & Recovery** — Soft delete prevents accidental loss with list and restore capabilities.
- **📦 Import & Export** — Export single notes to HTML or full vaults to `.zip` archives; import `.md`, directories, or `.zip` files seamlessly.

## 🚀 Installation & Quick Start

### ⚡ Instant Install

#### Linux / macOS (Shell)
```bash
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/install.sh | sh
```

#### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/tareqmy/ferronote/master/install.ps1 | iex
```

### 🍺 Homebrew (macOS / Linux)

```bash
# Direct install (recommended)
brew install tareqmy/tap/ferronote

# Or tap first:
brew tap tareqmy/tap
brew install ferronote
```

### 🦀 Other Package Managers

```bash
# Cargo (crates.io)
cargo install ferronote

# Cargo (direct from git)
cargo install --git https://github.com/tareqmy/ferronote

# Arch Linux (AUR)
yay -S ferronote
```

### 🗑️ Uninstalling

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/install.sh | UNINSTALL=true sh

# Windows (PowerShell)
$env:UNINSTALL='true'; irm https://raw.githubusercontent.com/tareqmy/ferronote/master/install.ps1 | iex
```

---

## 💻 CLI Commands & Options

| Command / Flag | Description | Example Usage |
| :--- | :--- | :--- |
| `ferronote` / `fnt` | Launch interactive TUI (supports `fnt` shortcut) | `ferronote` or `fnt` |
| `-d`, `--dir <PATH>` | Specify custom notes directory | `fnt --dir ~/notes` |
| `-i`, `--import <PATH>` | Import `.md`, `.txt`, directory, or `.zip` archive | `fnt --import ~/backup/notes.zip` |
| `-e`, `--export <PATH>` | Export vault to `.zip` or single note to `.html` | `fnt --export ~/exports/note.html` |
| `--trash` | List soft-deleted notes in trash | `fnt --trash` |
| `--restore <FILENAME>` | Restore a trashed note by filename | `fnt --restore "deleted-note.md"` |
| `-h`, `--help` | Display CLI help documentation | `fnt --help` |
| `-V`, `--version` | Display version information | `fnt --version` |

## ⌨️ Keybindings

| Key                 | Action                                              |
|---------------------|-----------------------------------------------------|
| `/` or `Ctrl+L`     | Focus search bar                                    |
| `Tab` / `Shift+Tab` | Cycle focus (Search → Note List → Editor)           |
| `Enter`             | Open selected note / Create new / Follow wiki-link  |
| `↑` / `↓`           | Navigate note list / settings items                 |
| `PageUp` / `PageDown` | Page jump note list                               |
| `Ctrl+S`            | Force save current note                             |
| `Ctrl+D`            | Delete note (moves to trash)                        |
| `Ctrl+N`            | New note                                            |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo in editor                               |
| `Ctrl+P`            | Toggle Interactive Settings Overlay                 |
| `Ctrl+V`            | Toggle About Application Overlay                    |
| `?`                 | Toggle Help Overlay                                 |
| `Mouse Click`       | Focus panel (Search/List/Editor) / select note item |
| `Esc`               | Close overlay / Back to search                      |
| `Ctrl+Q`                   | Quit application                                    |

## 🏗️ Architecture

Ferronote uses **The Elm Architecture (TEA)** adapted for Rust:

```
Event → Action → Update State → Render
```

- **Model**: Application state (`app.rs`)
- **Update**: Action processing and state transitions (`action.rs`)
- **View**: Ratatui widget rendering

Built with:
- [`ratatui`](https://ratatui.rs/) + [`crossterm`](https://docs.rs/crossterm) — Terminal UI
- [`tui-textarea`](https://docs.rs/tui-textarea) — Text editing
- [`fuzzy-matcher`](https://docs.rs/fuzzy-matcher) — Fuzzy search engine
- [`tokio`](https://tokio.rs/) — Async runtime & file watcher
- [`clap`](https://docs.rs/clap) — CLI arguments
- [`serde`](https://serde.rs/) — JSON configuration & metadata

## 📁 Project Structure

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

## 📄 License

MIT

