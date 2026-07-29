# Ferronote — Style Guide & UI Conventions

## Layout

The UI consists of three main regions arranged vertically:

```
┌────────────────────────────────────────────┐
│  🔍 Search / Create                        │  ← Search Bar (always visible)
├────────────────────────────────────────────┤
│  ┌──────────────┐ ┌──────────────────────┐ │
│  │ Note List    │ │ Note Editor          │ │  ← Main content area
│  │              │ │                      │ │
│  │ > Meeting..  │ │  # Meeting Notes     │ │
│  │   Ideas      │ │                      │ │
│  │   Todo       │ │  Today we discussed  │ │
│  │              │ │  the following...    │ │
│  └──────────────┘ └──────────────────────┘ │
├────────────────────────────────────────────┤
│  [Ctrl+N] New  [Ctrl+D] Del  [Ctrl+Q] Quit│  ← Status Bar (keybinding hints)
└────────────────────────────────────────────┘
```

### Proportions
- **Search bar**: 3 lines (1 border top, 1 input, 1 border bottom)
- **Note list**: 30% of remaining width (min 20 cols, max 40 cols)
- **Editor**: Remaining space
- **Status bar**: 1 line

## Color Palettes & Themes

Ferronote supports dynamic theme switching via `ThemePalette` (`src/theme.rs`):

- **default**: Standard blue active borders, yellow search highlights, dark gray selection.
- **gruvbox**: Warm retro palette with yellow active borders (`#FABD2F`), green highlights (`#B8BB26`), and teal selection (`#458588`).
- **nord**: Cool arctic palette with cyan active borders (`#88C0D0`), green highlights (`#A3BE8C`), and deep blue selection (`#5E81AC`).
- **dracula**: Vibrant dark palette with purple active borders (`#BD93F9`), green highlights (`#50FA7B`), and magenta accent (`#FF79C6`).

## Interaction Patterns & Overlays

### Focus Model
The app has three focusable main regions:
1. **Search Bar** — Default focus on startup (`/` or `Ctrl+L`)
2. **Note List** — Arrow keys after typing in search
3. **Editor** — Enter on a note to start editing

Focus flows: `Search → List → Editor → (Esc) → Search` (cycled with `Tab` / `Shift+Tab` or clicked with Mouse).

### Settings Overlay (`Ctrl+P`)
- Rendered as a centered modal popup box overlaying the main layout.
- Options: Theme, Tab Size, Sidebar Width, Default Sort, Auto-Purge Days, Auto-Save Delay, Show Modified Time, Word Wrap.
- Controls: `↑`/`↓` to select setting item, `←`/`→` or `Space`/`Enter` to cycle value options, `Esc` to save & close.

### About Overlay (`Ctrl+V`)
- Rendered as a centered modal popup box.
- Displays app version (`v1.0.1`), creator (`Tareq Mohammad Yousuf` / `tareqmy.com`), contact email (`tareq.y@gmail.com`), GitHub repo URL, and license.
- Pressing `Esc` or any key closes the modal.

### Search Behavior
- As the user types, the note list filters in real-time using Skim fuzzy matching.
- Searching with `#tag` prefix filters notes containing that `#tag`.
- Matching characters in titles are highlighted with active theme accent color.
- If 0 title matches exist, the note list offers a `+ Create '[query]'` prompt.

### Editor Behavior
- Auto-saves debounced at 1000ms after user stops typing.
- Pressing `Enter` on a `[[Note Title]]` wiki-link navigates directly to that note.
- `Ctrl+Z` / `Ctrl+Y` triggers undo/redo in editor buffer.

