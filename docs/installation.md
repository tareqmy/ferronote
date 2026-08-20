# 🚀 Installation & Quick Start

Ferronote can be installed via instant installation scripts, Homebrew, Cargo, or Nix.

---

## ⚡ Instant Install

### Linux / macOS (Shell)
```bash
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/install.sh | sh
```
*Installs binary and `fn` / `fnt` shortcuts to `/usr/local/bin` (or `~/.local/bin` if non-root). To install to a custom directory, pass `INSTALL_DIR`:*
```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/install.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/install.ps1 | iex
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

### Nix Flake
```bash
nix run github:tareqmy/ferronote
```

---

## 🗑️ Uninstalling

To completely remove Ferronote from your system:

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/uninstall.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/tareqmy/ferronote/master/scripts/uninstall.ps1 | iex
```

### Homebrew / Cargo
```bash
brew uninstall ferronote
# or
cargo uninstall ferronote
```
