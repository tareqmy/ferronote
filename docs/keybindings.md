# ⌨️ Keybindings Reference

Ferronote is built from the ground up for keyboard efficiency.

---

## ⌨️ Shortcuts & Hotkeys

| Key | Action | Context |
| :--- | :--- | :--- |
| `/` or `Ctrl+L` | Focus search bar | Global |
| `Tab` / `Shift+Tab` | Cycle panel focus (Search → Note List → Editor) | Global |
| `Enter` | Open selected note / Create new note / Follow wiki-link or web URL | Global |
| `↑` / `↓` | Navigate note list / settings items | List / Overlay |
| `PageUp` / `PageDown` | Page jump note list / settings items | List / Overlay |
| `Home` / `End` | Jump to start/end of note list / settings items | List / Overlay |
| `Ctrl+S` | Force save current note | Editor |
| `Ctrl+D` | Delete active note (prompts for y/n, moves to trash) | Global |
| `Ctrl+R` | Rename active note | Global |
| `Ctrl+N` | Create new note | Global |
| `Ctrl+K` | Toggle Pin / Bookmark note (or 'p' in Note List) | Global / List |
| `Ctrl+O` | Open active note in external editor (configured in Settings, or `$VISUAL` / `$EDITOR`) | Global |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo text edits | Editor |
| `Ctrl+P` | Toggle Interactive Settings Overlay | Global |
| `Ctrl+V` | Toggle About Application Overlay | Global |
| `?` | Toggle Help Overlay | Global |
| `Mouse Click` | Focus panel / select note item | Global |
| `Mouse Drag` | Drag panel divider to resize notes list and content (20%-40%) | Global |
| `Esc` | Close overlay / Clear search bar / Return focus | Global / Overlay |
| `Ctrl+Q` | Quit application | Global |

---

## ⚡ Vim Keybindings (Experimental)

These keybindings are active only when focused on the **Editor** in **View** mode. Press `i`, `a`, or `o` to enter **Insert** mode for normal typing.

| Key | Action | Context |
| :--- | :--- | :--- |
| `h`, `j`, `k`, `l` | Move cursor left, down, up, right | View mode |
| `w`, `b`, `W`, `B` | Move word forward, backward, by whitespace | View mode |
| `e`, `ge` | Move to end of word (forward/backward) | View mode |
| `0`, `^`, `$` | Move to beginning, first non-blank, end of line | View mode |
| `+`, `-` | Move to first non-blank of next/prev line | View mode |
| `gg`, `G` | Move to beginning, end of file | View mode |
| `%` | Jump to matching brace/bracket | View mode |
| `dd`, `dNd` | Delete current line / N lines | View mode |
| `yy`, `yNy` | Yank (copy) current line / N lines | View mode |
| `p`, `P` | Paste after, before cursor | View mode |
| `u`, `Ctrl+R` | Undo, Redo | View mode |
| `/` | Search locally in file (regex supported) | View mode |
| `n`, `N` | Next match, previous match | View mode |
| `x` | Delete character under cursor | View mode |
| `i`, `a`, `o` | Enter Insert mode | View mode |
| `v`, `V` | Enter Visual mode (char / line) | View mode |
| `Esc` | Exit back to View mode / Clear local search | Insert / View mode |

---

## 🧭 Panel Navigation Flow

```
+------------------------------------------+
|  Search Bar  (Focus: SearchBar)          |
+------------------------------------------+
|  Note List   |  Editor                   |
|  (Focus:     |  (Focus:                  |
|   NoteList)  |   Editor)                 |
+------------------------------------------+
```

Press `Tab` or `Shift+Tab` to seamlessly switch active focus between the Search Bar, Note List, and Editor panels.
