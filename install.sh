#!/bin/bash
# Install script for memon.py

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MEMON_PY_PATH="$SCRIPT_DIR/memon.py"

# Check if memon.py exists
if [ ! -f "$MEMON_PY_PATH" ]; then
    echo "Error: memon.py not found in $SCRIPT_DIR"
    exit 1
fi

# Make memon.py executable
chmod +x "$MEMON_PY_PATH"

# Determine installation directory
INSTALL_DIR="$HOME/.local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
fi

# Create symlink in installation directory
SYMLINK_PATH="$INSTALL_DIR/memon"
ln -sf "$MEMON_PY_PATH" "$SYMLINK_PATH"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Adding $INSTALL_DIR to your PATH"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.bashrc"
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.zshrc"
    echo "Please run 'source ~/.bashrc' or 'source ~/.zshrc' to update your PATH"
fi

echo "memon has been installed successfully!"
echo "You can now run 'memon' from anywhere"
echo ""
echo "Usage: memon <process_name>"
echo "Example: memon chrome"