#!/usr/bin/env bash
set -e

REPO="duwei0227/mock-apis"
BINARY="mock"
ARCHIVE="mock-linux-x86_64.tar.gz"

# Determine install directory
if [ -w "/usr/local/bin" ] || [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')

if [ -z "$LATEST" ]; then
    echo "Error: could not fetch latest release." >&2
    exit 1
fi

echo "Downloading ${BINARY} ${LATEST}..."
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "https://github.com/${REPO}/releases/download/${LATEST}/${ARCHIVE}" \
    -o "$TMP/$ARCHIVE"

tar -xzf "$TMP/$ARCHIVE" -C "$TMP"
chmod +x "$TMP/$BINARY"
mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"

echo "Installed: $INSTALL_DIR/$BINARY"

# Remind user to add to PATH if using ~/.local/bin
if [ "$INSTALL_DIR" = "$HOME/.local/bin" ]; then
    if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
        echo ""
        echo "Add the following line to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo "Then run: source ~/.bashrc"
    fi
fi

echo "Done. Run: mock --help"
