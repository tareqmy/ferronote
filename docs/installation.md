# 🚀 Installation & Quick Start

Ferronote can be installed via instant installation scripts, Homebrew, Cargo, or AUR.

---

## ⚡ Instant Install

### Linux / macOS (Shell)
```bash
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/install.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/tareqmy/ferronote/master/install.ps1 | iex
```

---

## 🍺 Homebrew (macOS / Linux)

Install directly from the official tap:

```bash
# Direct install (recommended)
brew install tareqmy/tap/ferronote

# Or tap first and install:
brew tap tareqmy/tap
brew install ferronote
```

---

## 🦀 Other Package Managers

### Cargo (crates.io)
```bash
cargo install ferronote
```

### Cargo (Direct from Git)
```bash
cargo install --git https://github.com/tareqmy/ferronote
```

### Arch Linux (AUR)
```bash
yay -S ferronote
```

---

## 🗑️ Uninstalling

To completely remove Ferronote from your system:

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/install.sh | UNINSTALL=true sh
```

### Windows (PowerShell)
```powershell
$env:UNINSTALL='true'; irm https://raw.githubusercontent.com/tareqmy/ferronote/master/install.ps1 | iex
```

### Homebrew / Cargo
```bash
brew uninstall ferronote
# or
cargo uninstall ferronote
```
