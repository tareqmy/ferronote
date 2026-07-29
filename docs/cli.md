# 💻 CLI Commands & Options

Ferronote provides a flexible command-line interface (`ferronote` or shortcut `fnt`) for launching the TUI app, managing note vaults, importing/exporting, and managing trash.

---

## 🛠️ Command Summary

| Command / Flag | Description | Example Usage |
| :--- | :--- | :--- |
| `ferronote` / `fnt` | Launch interactive TUI | `ferronote` or `fnt` |
| `-d`, `--dir <PATH>` | Specify custom notes directory | `fnt --dir ~/notes` |
| `-i`, `--import <PATH>` | Import `.md`, `.txt`, directory, or `.zip` archive | `fnt --import ~/backup/notes.zip` |
| `-e`, `--export <PATH>` | Export vault to `.zip` or single note to `.html` | `fnt --export ~/exports/note.html` |
| `--trash` | List soft-deleted notes in trash | `fnt --trash` |
| `--restore <FILENAME>` | Restore a trashed note by filename | `fnt --restore "deleted-note.md"` |
| `-h`, `--help` | Display CLI help documentation | `fnt --help` |
| `-V`, `--version` | Display version information | `fnt --version` |

---

## 💡 Usage Examples

### Launching TUI with Custom Directory
```bash
fnt --dir ~/Documents/my-vault
```

### Importing Notes
Import a single Markdown file or a zip archive:
```bash
fnt --import ~/Downloads/notes-backup.zip
```

### Exporting Vault
Export full note vault to a zip archive:
```bash
fnt --export ~/Backups/vault-$(date +%Y%m%d).zip
```

### Managing Trashed Notes
List all soft-deleted notes:
```bash
fnt --trash
```

Restore a deleted note back to your active notes directory:
```bash
fnt --restore "project-ideas.md"
```
