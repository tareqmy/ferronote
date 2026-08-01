<p align="center">
  <img src="assets/logo.jpg" alt="Ferronote Logo" width="650" />
</p>

<p align="center">
  <b>Notes at the speed of thought</b> — a blazing-fast terminal note-taking app inspired by <a href="https://notational.net/">Notational Velocity</a>.
</p>

Ferronote is a Rust TUI application that brings the elegance and speed of Notational Velocity to the modern terminal. Search, create, and edit notes without ever touching the mouse.

---

## 📚 Documentation Index

Explore detailed documentation sections:

- 📖 **[Philosophy & Features](docs/features.md)** — Modeless design, plain text storage, tags, wiki links, and custom themes.
- 🚀 **[Installation & Quick Start](docs/installation.md)** — Install via Shell/PowerShell scripts, Homebrew, Cargo, or AUR.
- 💻 **[CLI Commands & Options](docs/cli.md)** — Command-line flags, custom directory flags, vault import/export, and trash management.
- ⌨️ **[Keybindings Reference](docs/keybindings.md)** — Full list of keyboard shortcuts, navigation, and panel focus controls.
- 🏗️ **[Architecture & Structure](docs/architecture.md)** — The Elm Architecture (TEA) in Rust, dependencies, and source tree layout.
- 🤝 **[Contributing Guide](CONTRIBUTING.md)** — Guidelines for opening issues and submitting pull requests.
- 🗺️ **[Development Roadmap](ROADMAP.md)** — Project vision and upcoming features.

---

## ✨ Key Features At A Glance

- **Modeless** — Search and note creation combined into a single bar.
- **Keyboard-first** — Control every aspect of the app without leaving your keyboard.
- **External Editor Support** — Press `Ctrl+O` to seamlessly edit any note in your preferred advanced terminal editor (configure explicitly via `Ctrl+P` Settings, or use `$VISUAL` / `$EDITOR` fallback).
- **Plain Text** — Notes stored as standard Markdown files in your directory of choice.
- **Tag & Backlink Support** — Categorize with `#tag` annotations and link via `[[Note Title]]`.
- **Custom Themes & Overlays** — Tailor visual aesthetics (`Ctrl+P`) and access help on demand.

👉 *Read more in the [Features Documentation](docs/features.md).*

---

## ⚡ Quick Install

```bash
# Linux / macOS (Shell)
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/install.sh | sh

# macOS / Linux (Homebrew)
brew install tareqmy/tap/ferronote
```

👉 *For Windows, Cargo, AUR, and uninstallation instructions, see the [Installation Guide](docs/installation.md).*

---

## 💻 Quick CLI Reference

```bash
# Launch interactive TUI (using shortcut fn or fnt)
fn

# Launch with custom notes directory
fn --dir ~/notes

# Import backup vault (.zip) or Markdown file
fn --import ~/backup.zip
```

👉 *For full CLI flags and usage examples, see the [CLI Documentation](docs/cli.md).*

---

## ⌨️ Essential Keybindings

| Key | Action |
| :--- | :--- |
| `/` or `Ctrl+L` | Focus search bar |
| `Esc` | Clear global search bar |
| `Tab` / `Shift+Tab` | Cycle focus (Search → Note List → Editor) |
| `Enter` | Open selected note / Create new note / Follow wiki-link |
| `Ctrl+O` | Open active note in external editor (configured in Settings, or `$VISUAL` / `$EDITOR`) |
| `Ctrl+P` | Toggle Interactive Settings Overlay |
| `Mouse Drag` | Drag panel divider to resize notes list and content (20%-40%) |
| `Ctrl+Q` | Quit application |

👉 *For complete shortcuts and overlay controls, view the [Keybindings Reference](docs/keybindings.md).*

---

## 📄 License

[MIT](LICENSE)
