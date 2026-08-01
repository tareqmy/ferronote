# ✨ Philosophy & Features

Ferronote is designed around speed, simplicity, and complete ownership of your data. Inspired by [Notational Velocity](https://notational.net/), it brings modal-less note search and creation to the terminal.

---

## 🎯 Design Philosophy

- **Modeless** — The search bar *is* the creation bar. Type to search; if no match exists, press `Enter` to create a new note immediately.
- **Keyboard-first** — Every feature and navigation path is accessible via hotkeys. No mouse needed.
- **Plain Text Freedom** — Notes are saved as plain Markdown files (`.md`) in a standard directory. Zero proprietary lock-in, zero database reliance.
- **Instant Speed** — Sub-millisecond fuzzy search powered by Rust, enabling instant lookups even across thousands of notes.

---

## 🌟 Key Features

### 🏷️ Tag Filtering
Organize notes dynamically using `#tag` syntax in note bodies. Filter notes in real-time by typing `#tag` into the search bar.

### 🔗 Wiki Links & Backlinks
Link notes effortlessly using `[[Note Title]]` syntax. Ferronote automatically detects backlinks across your vault and lets you navigate between linked notes with `Enter`.

### 📝 External Editor Support
While Ferronote ships with a blazing-fast builtin editor for quick capturing, you can press `Ctrl+O` at any time to seamlessly drop into your preferred external terminal editor. By default, Ferronote reads your `$VISUAL` or `$EDITOR` environment variables, but you can explicitly set your editor of choice (like `nvim`, `hx`, or `nano`) by typing it directly into the `Ctrl+P` Settings overlay. Ferronote intelligently suspends its UI while you edit and automatically reloads the note once you finish.

### 🎨 Custom Themes
Tailor your terminal environment with pre-built theme palettes:
- `default` (Nord-inspired dark palette)
- `gruvbox`
- `nord`
- `dracula`

### ⚙️ Interactive Settings & Panel Resizing
Press `Ctrl+P` to open the interactive settings overlay. Modify note directory, theme, auto-save interval, and display options on the fly.
Drag the panel divider boundary with your mouse to dynamically resize the notes list and content panels (clamped between 20% and 40%, defaulting to the 38% Golden Ratio split).

### 🗑️ Trash & Recovery
Deleted notes (`Ctrl+D`) are moved to a soft-delete trash directory, ensuring accidental deletions can be restored easily via CLI (`--restore`).

### 📦 Vault Import & Export
- **Export**: Export single notes to `.html` or full vaults to `.zip` archives.
- **Import**: Import `.md`, `.txt`, entire directories, or `.zip` archives into your vault seamlessly.
