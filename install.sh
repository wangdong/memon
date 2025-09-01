#!/bin/bash

# Memon Installation Script
# Installs memon to user's local bin directory

set -e

echo "Installing memon..."

# Check if Rust/Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo is not installed or not in PATH"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Install using cargo
echo "Building and installing memon..."
cargo install --path .

# Check if installation was successful
if command -v memon &> /dev/null; then
    echo "memon has been successfully installed!"
    echo "Location: $(which memon)"
    echo ""
    echo "Usage: memon <process_name>"
    echo "Example: memon firefox"
    echo "Help: memon --help"
else
    echo "Installation failed. Please check that ~/.cargo/bin is in your PATH"
    echo "Add this line to your shell profile (.bashrc, .zshrc, etc.):"
    echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
fi