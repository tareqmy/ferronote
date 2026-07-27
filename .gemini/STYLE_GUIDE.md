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

## Color Palette

Use a muted, terminal-friendly palette that works on both dark and light backgrounds:

| Element          | Dark Theme         | Light Theme        |
|------------------|--------------------|--------------------|
| Background       | Terminal default   | Terminal default   |
| Search highlight | Yellow (#FABD2F)   | Yellow (#D79921)   |
| Selected note    | Blue bg (#458588)  | Blue bg (#076678)  |
| Note title       | White/Bold         | Black/Bold         |
| Note snippet     | Gray (#928374)     | Gray (#7C6F64)     |
| Editor text      | White              | Black              |
| Status bar       | Dark gray bg       | Light gray bg      |
| Match highlight  | Green (#B8BB26)    | Green (#79740E)    |

## Interaction Patterns

### Focus Model
The app has three focusable regions:
1. **Search Bar** — Default focus on startup
2. **Note List** — Arrow keys after typing in search
3. **Editor** — Enter on a note to start editing

Focus flows: `Search → List → Editor → (Esc) → Search`

### Search Behavior
- As the user types, the note list filters in real-time using fuzzy matching
- Matching characters in titles are highlighted
- If 0 results, the status bar shows "Press Enter to create '[query]'"
- Search matches both title and content

### Editor Behavior
- Auto-save after 1 second of inactivity
- No explicit "save" action needed (but Ctrl+S force-saves immediately)
- Cursor position preserved when switching between notes
