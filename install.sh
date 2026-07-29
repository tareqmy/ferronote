#!/bin/sh
set -e

REPO="tareqmy/ferronote"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)  OS_TYPE="linux-amd64" ;;
  Darwin) OS_TYPE="macos-arm64" ;;
  *)
    echo "Error: Unsupported OS '$OS'. Please install via 'cargo install ferronote'."
    exit 1
    ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
if [ "$OS" = "Linux" ] && [ "$ARCH" != "x86_64" ]; then
  echo "Error: Unsupported Linux architecture '$ARCH'."
  exit 1
fi

FILE_NAME="ferronote-${OS_TYPE}.tar.gz"

echo "⚡ Installing Ferronote for ${OS} (${ARCH})..."

# Get latest release tag if VERSION is not specified
if [ -z "$VERSION" ]; then
  TAG="$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')"
  if [ -z "$TAG" ]; then
    TAG="v1.0.1"
  fi
else
  TAG="$VERSION"
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${FILE_NAME}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "📥 Downloading Ferronote ${TAG}..."
curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/$FILE_NAME"

echo "📦 Extracting archive..."
tar -xzf "$TMP_DIR/$FILE_NAME" -C "$TMP_DIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_DIR/ferronote" "$INSTALL_DIR/ferronote"
else
  echo "🔒 Sudo permissions required to install to $INSTALL_DIR..."
  sudo mv "$TMP_DIR/ferronote" "$INSTALL_DIR/ferronote"
fi

chmod +x "$INSTALL_DIR/ferronote"

echo "✨ Ferronote ${TAG} successfully installed to ${INSTALL_DIR}/ferronote!"
echo "Run 'ferronote' to start taking notes at the speed of thought."
