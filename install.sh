#!/bin/sh
set -e

REPO="nickagliano/hookplayer"
INSTALL_DIR="${HOOKPLAYER_INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS
OS=$(uname -s)
case "$OS" in
  Darwin) OS="macos" ;;
  Linux)  OS="linux" ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Detect arch
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)        ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Fetch latest release tag
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Could not determine latest release."
  exit 1
fi

ASSET="hookplayer-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"

echo "Installing hookplayer ${LATEST} (${OS}/${ARCH})..."

mkdir -p "$INSTALL_DIR"
curl -fsSL "$URL" -o "$INSTALL_DIR/hookplayer"
chmod +x "$INSTALL_DIR/hookplayer"

# Create default config if absent
CONFIG_DIR="$HOME/.config/hookplayer"
CONFIG_FILE="$CONFIG_DIR/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
  mkdir -p "$CONFIG_DIR"
  curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/config.toml" \
    -o "$CONFIG_FILE"
  echo "Created default config at $CONFIG_FILE"
fi

echo "hookplayer installed to $INSTALL_DIR/hookplayer"

# Warn if install dir is not in PATH
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Warning: $INSTALL_DIR is not in your PATH. Add it to your shell config." ;;
esac
