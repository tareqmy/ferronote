# Ferronote — Development Roadmap

> A phased plan to build a Notational Velocity replacement in Rust TUI.

---

## Phase 0 · Foundation (Milestone: "It runs") 
**Goal**: Scaffolded project that compiles, runs, and shows an empty TUI.

- [x] Initialize Cargo project with all dependencies
- [x] Set up `.gemini/AGENTS.md` (AI agent instructions)
- [x] Set up `.gemini/STYLE_GUIDE.md` (UI conventions)
- [x] Set up `.gemini/rules.md` (Gemini rules)
- [x] Create `README.md` with project overview
- [x] Create `LICENSE` (MIT)
- [x] Implement `tui.rs` — terminal init/restore with panic hook
- [x] Implement `event.rs` — async crossterm event polling loop
- [x] Implement `action.rs` — initial `Action` enum
- [x] Implement `app.rs` — skeleton App struct with run loop
- [x] Wire up `main.rs` — CLI parsing, terminal setup, run the app
- [x] Verify: app starts, shows blank screen, quits on `Ctrl+Q`

**Deliverable**: A running TUI that gracefully handles startup, rendering, and shutdown.

---

## Phase 1 · Note Storage (Milestone: "It reads and writes")
**Goal**: Read/write plain Markdown files from a configurable directory.

- [x] Implement `note_store.rs` — `NoteStore` struct
  - [x] `scan_directory()` — discover all `.md` files
  - [x] `load_note(path)` — read file contents into memory
  - [x] `save_note(path, content)` — write content to disk (atomic write via temp file)
  - [x] `create_note(title)` — create a new `.md` file
  - [x] `delete_note(path)` — delete with confirmation
  - [x] `rename_note(old, new)` — rename file on disk
- [x] Implement `config.rs` — load/save user config
  - [x] Notes directory path (default: `~/ferronotes/`)
  - [x] Config file location: `~/.config/ferronote/config.json`
- [x] Metadata sidecar: `.ferronote/metadata.json` in notes dir
  - [x] Track `created_at`, `modified_at` per note
- [x] Unit tests for all `NoteStore` operations

**Deliverable**: Rock-solid file I/O layer with tests. No data loss possible.

---

## Phase 2 · Core UI (Milestone: "It looks like Notational Velocity")
**Goal**: Three-panel layout with search bar, note list, and editor.

- [ ] Implement `components/search_bar.rs`
  - [ ] Text input widget with cursor
  - [ ] Real-time filtering as user types
  - [ ] Visual feedback: "Press Enter to create '[query]'" when 0 matches
- [ ] Implement `components/note_list.rs`
  - [ ] Scrollable list of note titles
  - [ ] Selected item highlighting
  - [ ] Fuzzy match character highlighting in titles
  - [ ] Sort by: last modified (default), title, created
- [ ] Implement `components/editor.rs`
  - [ ] Integrate `tui-textarea` for Markdown editing
  - [ ] Display note title as header
  - [ ] Line numbers (optional, toggleable)
- [ ] Layout manager
  - [ ] Three-panel split as defined in `STYLE_GUIDE.md`
  - [ ] Status bar with keybinding hints
  - [ ] Responsive: collapse note list if terminal too narrow
- [ ] Focus management
  - [ ] Tab / Esc cycling between Search → List → Editor

**Deliverable**: Fully navigable UI with all three panels rendering real note data.

---

## Phase 3 · Search Engine (Milestone: "It finds everything instantly")
**Goal**: Blazing-fast fuzzy search across titles and content.

- [ ] Integrate `fuzzy-matcher` (Skim algorithm)
  - [ ] Title matching with score-based ranking
  - [ ] Content matching (search body text)
  - [ ] Combined score: title matches weighted 3x over content
- [ ] In-memory index
  - [ ] Build on startup from `NoteStore`
  - [ ] Update incrementally on note create/edit/delete
- [ ] Search UX
  - [ ] Results update as-you-type (debounced at ~50ms)
  - [ ] Highlight matching characters in note list
  - [ ] Show content preview snippet with match context
- [ ] Performance benchmarks
  - [ ] Target: < 5ms for 10,000 notes
  - [ ] Add `criterion` benchmarks for search

**Deliverable**: Sub-millisecond fuzzy search that feels instant.

---

## Phase 4 · Unified Create Flow (Milestone: "Search IS create")
**Goal**: Implement the signature Notational Velocity behavior.

- [ ] When search query has no exact title match:
  - [ ] Show "Create new note: '[query]'" option at top of list
  - [ ] Enter creates note with that title and opens editor
- [ ] When search query matches an existing title exactly:
  - [ ] Auto-select and preview that note
  - [ ] Enter opens editor at that note
- [ ] Empty search state:
  - [ ] Show all notes sorted by last modified
  - [ ] First note auto-previewed in editor pane

**Deliverable**: The core Notational Velocity interaction model working end-to-end.

---

## Phase 5 · Auto-Save & Reliability (Milestone: "It never loses data")
**Goal**: Bulletproof data persistence.

- [ ] Auto-save
  - [ ] Save after 1 second of editor inactivity (debounced)
  - [ ] Save on note switch, search focus, and quit
  - [ ] Atomic writes (write to `.tmp`, then rename)
- [ ] Crash recovery
  - [ ] Detect orphaned `.tmp` files on startup → recover
  - [ ] Panic hook saves current editor buffer before exit
- [ ] File watching
  - [ ] Detect external changes to notes directory
  - [ ] Reload modified notes (with conflict resolution if editing)
- [ ] Undo/Redo
  - [ ] `tui-textarea` built-in undo support
  - [ ] Per-note undo history (in-memory, lost on quit)

**Deliverable**: Users can trust Ferronote with their notes. No data loss scenarios.

---

## Phase 6 · Polish & UX (Milestone: "It feels great to use")
**Goal**: Refine the experience until it feels effortless.

- [ ] Keybinding customization
  - [ ] Load custom keybindings from config
  - [ ] Display current bindings in status bar
- [ ] Color theme support
  - [ ] Dark theme (default)
  - [ ] Light theme
  - [ ] Custom theme via config
- [ ] Note metadata display
  - [ ] Created/modified timestamps in status bar
  - [ ] Word count
  - [ ] Character count
- [ ] Smooth scrolling & transitions
  - [ ] Animated cursor in note list
  - [ ] Smooth scroll for long note lists
- [ ] Help overlay
  - [ ] `?` key shows keybinding reference
  - [ ] First-run welcome/tutorial

**Deliverable**: A polished, delightful TUI that users *enjoy* using.

---

## Phase 7 · Advanced Features (Milestone: "Power user ready")
**Goal**: Features that differentiate Ferronote beyond Notational Velocity.

- [ ] **Tags**
  - [ ] Parse `#tag` from note content
  - [ ] Filter by tag in search
  - [ ] Tag autocomplete
- [ ] **Markdown preview**
  - [ ] Inline syntax highlighting in editor
  - [ ] Toggle-able rendered preview pane
- [ ] **Note linking**
  - [ ] `[[wiki-style]]` links between notes
  - [ ] Navigate links with Enter
  - [ ] Backlinks panel (which notes link to this one?)
- [ ] **Trash / Soft delete**
  - [ ] Move to `.ferronote/trash/` instead of permanent delete
  - [ ] Restore from trash
  - [ ] Auto-purge after 30 days
- [ ] **Export**
  - [ ] Export single note as HTML
  - [ ] Export all notes as a zip

**Deliverable**: Features that make Ferronote a daily-driver for power users.

---

## Phase 8 · Distribution & Community (Milestone: "Others can use it")
**Goal**: Make Ferronote easy to install and contribute to.

- [ ] **Packaging**
  - [ ] `cargo install ferronote` (publish to crates.io)
  - [ ] Homebrew formula
  - [ ] AUR package
  - [ ] Nix flake
  - [ ] Pre-built binaries (GitHub Releases via `cargo-dist`)
- [ ] **CI/CD**
  - [ ] GitHub Actions: `cargo fmt --check`, `cargo clippy`, `cargo test`
  - [ ] Cross-compilation matrix (Linux, macOS, Windows)
  - [ ] Automated release workflow
- [ ] **Documentation**
  - [ ] `man` page
  - [ ] Website / landing page
  - [ ] `CONTRIBUTING.md`
  - [ ] Architecture decision records (ADRs)
- [ ] **Community**
  - [ ] Issue templates
  - [ ] PR template
  - [ ] Code of Conduct

**Deliverable**: Ferronote is installable in one command and welcoming to contributors.

---

## Stretch Goals (Unprioritized)

- [ ] 🔄 **Sync** — Optional sync via git, Syncthing, or custom protocol
- [ ] 🔒 **Encryption** — Per-note or vault-level encryption at rest
- [ ] 🧩 **Plugin system** — Lua or WASM-based extensibility
- [ ] 🌐 **Web clipper** — Companion CLI to save URLs as notes
- [ ] 📱 **Companion mobile app** — Read-only viewer synced via filesystem
- [ ] 🤖 **AI integration** — Optional LLM-powered summarization, tagging, linking
- [ ] ⚡ **Vim keybindings** — Modal editing mode for vim users
- [ ] 📊 **Daily notes / Journal mode** — Auto-create daily notes template

---

## Guiding Principles for the Roadmap

1. **Each phase is shippable** — every milestone produces a usable (if minimal) tool.
2. **Phase order matters** — later phases depend on earlier ones being solid.
3. **Resist skipping ahead** — don't build Phase 7 features on Phase 0 foundations.
4. **Test as you go** — every phase includes its own testing requirements.
5. **Performance is not optional** — benchmark early, benchmark often.
