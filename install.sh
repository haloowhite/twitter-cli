#!/bin/bash
set -e

REPO="haloowhite/twitter-cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)  OS="linux" ;;
  darwin) OS="darwin" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *)             echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

ASSET="x-${OS}-${ARCH}.tar.gz"
echo "Detected: ${OS}/${ARCH}"
echo "Downloading ${ASSET}..."

# Get latest release download URL
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/$ASSET"
tar xzf "$TMP/$ASSET" -C "$TMP"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP/x" "$INSTALL_DIR/x"
else
  echo "Installing to $INSTALL_DIR (requires sudo)..."
  sudo mv "$TMP/x" "$INSTALL_DIR/x"
fi

chmod +x "$INSTALL_DIR/x"
echo "Installed x to $INSTALL_DIR/x"
echo ""
echo "Run 'x auth --browser chrome' to authenticate."
