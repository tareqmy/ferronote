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

- [x] Implement `components/search_bar.rs`
  - [x] Text input widget with cursor
  - [x] Real-time filtering as user types
  - [x] Visual feedback: "Press Enter to create '[query]'" when 0 matches
- [x] Implement `components/note_list.rs`
  - [x] Scrollable list of note titles
  - [x] Selected item highlighting
  - [x] Fuzzy match character highlighting in titles
  - [x] Sort by: last modified (default), title, created
- [x] Implement `components/editor.rs`
  - [x] Integrate `tui-textarea` for Markdown editing
  - [x] Display note title as header
  - [x] Line numbers (optional, toggleable)
- [x] Layout manager
  - [x] Three-panel split as defined in `STYLE_GUIDE.md`
  - [x] Status bar with keybinding hints
  - [x] Responsive: collapse note list if terminal too narrow
- [x] Focus management
  - [x] Tab / Esc cycling between Search → List → Editor

**Deliverable**: Fully navigable UI with all three panels rendering real note data.

---

## Phase 3 · Search Engine (Milestone: "It finds everything instantly")
**Goal**: Blazing-fast fuzzy search across titles and content.

- [x] Integrate `fuzzy-matcher` (Skim algorithm)
  - [x] Title matching with score-based ranking
  - [x] Content matching (search body text)
  - [x] Combined score: title matches weighted 3x over content
- [x] In-memory index
  - [x] Build on startup from `NoteStore`
  - [x] Update incrementally on note create/edit/delete
- [x] Search UX
  - [x] Results update as-you-type (debounced at ~50ms)
  - [x] Highlight matching characters in note list
  - [x] Show content preview snippet with match context
- [x] Performance benchmarks
  - [x] Target: < 5ms for 10,000 notes
  - [x] Add `criterion` benchmarks for search

**Deliverable**: Sub-millisecond fuzzy search that feels instant.

---

## Phase 4 · Unified Create Flow (Milestone: "Search IS create")
**Goal**: Implement the signature Notational Velocity behavior.

- [x] When search query has no exact title match:
  - [x] Show "Create new note: '[query]'" option at top of list
  - [x] Enter creates note with that title and opens editor
- [x] When search query matches an existing title exactly:
  - [x] Auto-select and preview that note
  - [x] Enter opens editor at that note
- [x] Empty search state:
  - [x] Show all notes sorted by last modified
  - [x] First note auto-previewed in editor pane

**Deliverable**: The core Notational Velocity interaction model working end-to-end.

---

## Phase 5 · Auto-Save & Reliability (Milestone: "Bulletproof Data")
**Goal**: Ensure notes are never lost.

- [x] Auto-save
  - [x] Save after 1 second of editor inactivity (debounced)
  - [x] Save on note switch, search focus, and quit
  - [x] Atomic writes (write to `.tmp`, then rename)
- [x] Crash recovery
  - [x] Detect orphaned `.tmp` files on startup -> recover
  - [x] Panic hook saves current editor buffer before exit (Optional/advanced?)
- [x] File watching
  - [x] Detect external changes to notes directory
  - [x] Reload modified notes (with conflict resolution if editing)
- [x] Undo/Redo
  - [x] `tui-textarea` built-in undo support
  - [x] Per-note undo history (in-memory, lost on quit)

**Deliverable**: Users can trust Ferronote with their notes. No data loss scenarios.

---

## Phase 6 · Polish & TUI Interactions (Milestone: "It feels great to use")
**Goal**: Refine the experience until it feels effortless.

- [x] Dynamic status bar (shows keybinds based on focus state)
- [x] Keybinds for deleting note (`Ctrl+D` from search or list)
- [x] Code cleanup and linting
- [x] Note metadata display
  - [x] Word count
  - [x] Character count
- [x] Smooth scrolling & transitions
  - [x] Skipped: opted for instant snapping NV style for speed
- [x] Help overlay
  - [x] `?` key shows keybinding reference

**Deliverable**: A polished, delightful TUI that users *enjoy* using.

---

## Phase 7 · Advanced Features (Milestone: "Power user ready")
**Goal**: Features that differentiate Ferronote beyond Notational Velocity.

- [x] **Tags**
  - [x] Parse `#tag` from note content
  - [x] Filter by tag in search
- [x] **Markdown preview**
  - [x] Syntax-aware HTML rendering export
- [x] **Note linking**
  - [x] `[[wiki-style]]` links between notes
  - [x] Navigate links with Enter
  - [x] Backlinks detection & status counter
- [x] **Trash / Soft delete**
  - [x] Move to `trash/` instead of permanent delete
  - [x] List & Restore from trash (`--trash` / `--restore`)
  - [x] Purge trash support
- [x] **Export**
  - [x] Export single note as HTML
  - [x] Export all notes as a zip archive
- [x] **Import**
  - [x] Import single `.md` / `.txt` file or directory via CLI (`--import`)
  - [x] Import notes from a `.zip` archive
- [x] **Settings & Configuration**
  - [x] Expanded `config.json` schema (`default_extension`, `auto_save_delay_ms`, `tab_size`, `sidebar_width_percent`, `theme`, `default_sort`, `auto_purge_days`)
  - [x] Interactive Settings TUI Overlay (`Ctrl+P`)
  - [x] Auto-upgrading backward-compatible config loader (`serde(default)`)

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
  - [x] `CONTRIBUTING.md`
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
