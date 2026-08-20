# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Site: the docs landing page now carries the Ferronote branding — logo wordmark in the nav, icon in the hero, favicon/social-preview meta, and the rust-orange/cream brand palette for accents.

## [1.2.10] - 2026-08-20

### Added
- Vim: system clipboard integration — yanks (`yy`, `Y`, `yNy`, `yw`, `yiw`, `y$`, visual `y`) also copy to the OS clipboard, so yanked text can be pasted into other applications.
- Vim: `p`/`P` paste text copied in another application since the last yank, falling back to the internal buffer otherwise — `dd` + `p` to move lines keeps working, and deletes never clobber the system clipboard.
- CI: GitHub Pages workflow deploys the `docs/` site to https://tareqmy.github.io/ferronote/ on pushes to `master`.

## [1.2.9] - 2026-08-18

### Added
- Vim: full operator grammar — `dw`, `diw`, `d$`, `D`, `cw`, `ciw`, `cc`, `C`, `yw`, `yiw`, `y$`, `Y` — with counts on line operations.
- Vim: in-line character search `f`/`t`/`F`/`T` with `;` repeat and `,` reverse.
- Vim: `r` (replace character) and `J` (join lines).
- Vim: Visual mode — `v` (character-wise) and `V` (line-wise) selections with `d`/`x`, `y`, `c` acting on the selection; works under word wrap.
- Vim: `Ctrl+D`/`Ctrl+U` half-page and `Ctrl+F` full-page scrolling in View mode.
- UI: mode-aware status bar hints when the editor is focused (View / Visual / Insert).

### Changed
- Keys: `Ctrl+D` (delete note) and `Ctrl+R` (rename) now apply from the Search bar and Note List; in the editor's View mode those keys scroll and redo, per the Vim layer.
- UI: pinned notes are marked with an ASCII `[P]` prefix instead of the pin emoji, which has inconsistent width across terminals.
- Internal: Vim key handling extracted into a dedicated state machine module (`src/vim.rs`).
- CI: Rust toolchain pinned to 1.97.0 via `rust-toolchain.toml` so lint results match across CI and local builds.

### Fixed
- UI: pinned notes' modified timestamp no longer overflows the note list border.
- Lint: `question_mark` warning from clippy 1.97 in settings shortcut matching.

## [1.2.8] - 2026-08-17

### Fixed
- Input: Replace the identical-keystroke debounce with key-press event filtering, so held keys auto-repeat at full speed and fast double letters (e.g. "ll") are no longer dropped.
- CI: Run the lint/test workflow on `master` (it previously targeted `main` and never ran).

### Changed
- New logo and brand assets.
- Code cleanup: resolve all clippy lints and rustfmt drift across the workspace.

## [1.2.7] - 2026-08-04

### Fixed
- Input: Debounce identical keystrokes to prevent flickering.
- UI: Restrict delete confirmation to `y/n` and `Esc`, ignoring other keystrokes.

## [1.2.6] - 2026-08-03

### Changed
- Release version 1.2.6

## [1.2.5] - 2026-08-02

### Added
- feat: Add Open URLs feature to allow opening web links under the cursor with `Enter` in View mode.
- docs: Add automated Criterion performance benchmark results to documentation.

## [1.2.4] - 2026-08-02

### Changed
- chore: sync workspace member versions and add READMEs

## [1.2.3] - 2026-08-02

### Fixed
- fix: publish workspace members to crates.io and add required metadata

## [1.2.2] - 2026-08-02

### Changed
- docs: sync keybindings, default notes, and help popup with 1.2.1 features

## [1.2.1] - 2026-08-01

### Added
- feat: implement missing `/` shortcut to focus global search bar
- feat: add case-insensitive regex search support for local editor search

### Changed
- refactor: change `Esc` behavior to clear global search bar and exit overlays
- refactor: dynamically size help popup based on content height
- refactor: use different border color for editor in edit vs view mode
- docs: update README, keybindings, and default notes to reflect new `Esc` and Vim behaviors
- chore: fix compilation warnings for deprecated layout methods

### Fixed
- fix: resolve default note generation overwriting existing files
- fix: correct note content snippet generation in search results handling multi-byte characters

## [1.1.7] - 2026-08-01

### Added
- feat: convert external editor setting to free-form text input
- feat: add external editor selection to settings overlay
- feat: add support for external editor and remove builtin formatting shortcuts
- feat: add mouse capture toggle to settings to allow native text selection
- feat: implement note renaming via UI with Ctrl+R
- feat: support mouse scroll on editor content
- feat: support PageUp/PageDown and Home/End in settings popup
- feat: add Shift+Tab (BackTab) support to cycle focus backwards
- feat: ensure search bar always inserts 1 create note prompt at top for non-empty queries
- feat: default editor to view mode with shortcut to toggle edit mode
- feat: eliminate shortcut overlaps and add non-overlapping formatting shortcuts
- feat: curtail long note titles in list, enable help popup scrolling, and set default show_modified_time to false

### Changed
- docs: add auto-generated changelog and update agent rules
- docs: sync docs and help menus with external editor settings option
- docs: add external editor support to features.md
- refactor: disable wrap around in note list navigation

### Fixed
- fix: show cursor pointer in editor view mode
- fix: truncate long note names in delete confirmation popup

## [1.1.6] - 2026-07-31

### Added
- feat: add mouse drag resizing for notes list and content panel

### Changed
- docs: update README, keybindings, man page, and Help overlay for mouse drag resizing

### Other
- release v1.1.6

## [1.1.5] - 2026-07-30

### Added
- docs: add Ctrl+B shortcut to UI and default note
- feat: add shortcut to toggle notes list visibility
- feat: add reverse sorting options
- feat: show sort order in notes list title
- feat: dynamically wrap status bar shortcuts
- feat: Add confirmation popup for note deletion

### Changed
- updated roadmap

### Fixed
- fix: immediately apply sorting when changed in settings
- fix: apply configured sort order in notes list

### Other
- chore: bump version to 1.1.5

## [1.1.4] - 2026-07-30

### Added
- feat(installer): add modular install and uninstall scripts following gitwig pattern
- ci: update CI/CD workflows and add cargo-deny config following gitwig pattern

### Changed
- docs: update README, installation guide, and scripts to support fn and fnt shortcuts

### Other
- release v1.1.4

## [1.1.3] - 2026-07-30

### Added
- chore: prepare release v1.1.2 and fix additional clippy warnings

### Other
- release v1.1.3

## [1.1.2] - 2026-07-30

### Fixed
- chore: prepare release v1.1.1 and fix clippy warnings

### Other
- release v1.1.2

## [1.1.1] - 2026-07-30

### Added
- feat: add auto-updater UI and background checker

### Other
- release v1.1.1
- ci: upgrade github actions to resolve Node.js 20 deprecation warning

## [1.0.9] - 2026-07-30

### Fixed
- Fix install script API rate limit and Linux glibc compatibility

### Other
- release v1.0.9
- Centralize versioning to use .version file as the single source of truth

## [1.0.8] - 2026-07-29

### Fixed
- fix(install): default install.sh to ~/.local/bin without requiring sudo

### Other
- chore(release): bump version to 1.0.8
- docs: split README into modular documentation files

## [1.0.7] - 2026-07-29

### Added
- ci: add publish-homebrew job to release workflow

### Other
- chore(release): bump version to 1.0.7

## [1.0.6] - 2026-07-29

### Changed
- docs: update Homebrew installation instructions

### Other
- release v1.0.6

## [1.0.5] - 2026-07-29

### Added
- fix(cargo): add default-run = "ferronote" to Cargo.toml
- feat(cli): add built-in 'fnt' binary shortcut

### Other
- release v1.0.5

## [1.0.4] - 2026-07-29

### Added
- fix(ci): add --allow-dirty to cargo publish and ignore release-assets
- feat: add .version file single source of truth and automated bump-version script

### Fixed
- fix(ci): fix step-level if condition syntax in pipeline.yml

### Other
- style(fmt): format rust source code to pass cargo fmt check
- release v1.0.4

## [1.0.3] - 2026-07-29

### Other
- chore(release): bump version to v1.0.3
- ci: simplify CI/CD to a single release pipeline workflow triggered on tag push
- ci: consolidate GitHub Actions into unified CI and Release workflows

## [1.0.2] - 2026-07-29

### Added
- feat(installer): add uninstall option to shell and powershell scripts
- feat(installer): add Windows PowerShell irm install script
- feat(installer): add curl to sh instant installation script

### Changed
- docs: update Cargo installation instructions for crates.io and direct git repo install

### Other
- chore(release): bump version to v1.0.2
- docs: organize installation, quick start, and CLI options in README.md

## [1.0.1] - 2026-07-29

### Added
- docs: add Architecture Decision Records (ADR 0001-0003)
- docs: add HTML landing page for GitHub Pages
- docs: add Unix man page for ferronote(1)
- ci: add automated crates.io release publishing workflow
- ci: add cross-compilation matrix workflow for Linux, macOS, and Windows
- ci: add GitHub Actions workflow for fmt, clippy, and test suite
- chore(packaging): add cargo-dist configuration and GitHub release workflow for pre-built binaries
- chore(packaging): add Nix flake for Nix package management
- chore(packaging): add AUR PKGBUILD for Arch Linux packaging
- chore(packaging): add Homebrew formula for Ferronote
- fix(app): add missing keyboard shortcuts, fix search tiebreaker, and update docs
- feat: add note list position setting and right-align modified info in note list
- feat: clear search bar when Esc key is pressed
- feat: add default Lorem Ipsum note with backlink from welcome note
- feat(editor): add PageUp and PageDown cursor page jump support
- feat(ui): add top header bar with application title on left and version on right
- feat(editor): add configurable word wrap setting enabled by default
- feat(ui): refine note list mouse click targeting with scroll offset and item line height support
- feat(ui): add mouse click support for panel selection and note item selection
- feat(settings): add setting to enable/disable last modified time in notes list
- feat(ui): add About Application modal (F1) and synchronize across docs, welcome note, status bar, and help overlay
- docs: add mandatory rule for synchronizing help overlay, docs, default note, and status bar when shortcuts are added
- feat: display last modified timestamps in note list items and status bar
- feat: ensure default welcome note is updated on every application launch
- docs: add rule to update default welcome note whenever documentation changes
- docs: add standalone square app icon assets
- docs: add project logo branding
- test: add comprehensive unit and integration test suite
- Add Settings & Configuration goal to Phase 7 in ROADMAP.md
- Add Import feature goal to Phase 7 in ROADMAP.md
- Add PageUp, PageDown, Home, and End navigation for Notes panel
- feat: implement phase 2 core UI (three-panel layout, focus)
- feat: implement phase 1 note storage layer
- feat: implement phase 0 foundation (TUI, events, app state)
- chore: add initial agent skills for project guidelines
- feat: scaffold project with dependencies, AI agent instructions, and roadmap

### Changed
- docs: update creator full name to Tareq Mohammad Yousuf
- refactor(shortcuts): change About overlay shortcut to Ctrl+V and remove F2 from settings overlay
- docs: update and synchronize all project documentation and doc comments
- docs: update official square app icon from new design
- docs: update official project logo
- Update Settings shortcut to F2 and Ctrl+P to avoid terminal emulator intercept
- Update ROADMAP.md with completed Phase 7 items

### Fixed
- fix(ci): fix release workflow permissions and artifact staging concurrency
- fix: improve wiki-link backlink matching and cross-link default notes
- fix(ui): pass show_modified_time to NoteList::click_at to accurately calculate item line heights when timestamps are disabled
- fix(editor): synchronize cursor position when word_wrap is enabled using map_cursor_to_wrapped and CursorMove::Jump
- fix(ui): render status bar at the bottom row (main_layout[3])
- fix(ui): increase Help Overlay popup width to 70
- fix(ui): align help overlay keybindings in a structured left-aligned tabular layout
- Fix status bar separator layout and prevent duplicate separators
- Validate and fix Makefile targets, clippy warnings, and benchmark compilation
- Fix duplicate note creation: editor.current_note now stores filename instead of title
- Fix infinite recursion stack overflow when saving note

### Other
- chore(release): bump version to v1.0.1
- chore(packaging): configure Cargo.toml metadata for crates.io publishing
- docs: remove Ctrl+, as settings shortcut reference, preserving F2 and Ctrl+P
- Implement ThemePalette dynamically mapping default, gruvbox, nord, and dracula themes
- Make Settings overlay interactive with left-aligned tabular layout
- Support Ctrl+, alongside F2 and Ctrl+P for Settings Overlay
- Implement Phase 7 Settings & Configuration and interactive TUI overlay (Ctrl+,)
- Remove background color from status bar and restore clean terminal default theme
- Improve theme colors and status bar keybinding contrast
- Mark Phase 7 as fully completed in ROADMAP.md
- Phase 7 Goal D: Implement Export (--export) to HTML and ZIP archive
- Phase 7 Goal C: Implement Trash listing, restoring (--restore), and purging
- Phase 7 Goal B: Implement Backlinks detection and status bar counter
- Phase 7 Goal A: Implement Note Import (--import) for files, directories, and zip archives
- Auto-create onboarding Welcome note on first launch
- Phase 7 Step 3: Implement Wiki-Links and Navigation
- Consolidate all Ferronote data under ~/.ferronote
- Phase 7 Step 2: Implement Tag Extraction and Search Filtering
- Phase 7 Step 1: Implement Soft Delete (Trash)
- Remove leftover test_app.rs scratch file
- Implement Phase 7: Soft delete, tags, and wiki-links
- Implement Help Overlay and Metadata
- Implement Phase 6: Polish & TUI Interactions
- Implement Phase 5: Auto-Save & Reliability
- Implement Phase 4: Unified Create Flow
- Implement Phase 3: Search Engine
- docs: mark Phase 0 goals as complete in roadmap

