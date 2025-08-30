#!/bin/bash
# Simple installation script for memon-qwen

set -e

# Create a virtual environment
python3 -m venv venv

# Activate the virtual environment
source venv/bin/activate

# Upgrade pip
pip install --upgrade pip

# Install dependencies
pip install psutil rich pytest

# Install memon-qwen in development mode
pip install -e .

echo "memon-qwen has been installed successfully in a virtual environment!"
echo "To use it, first activate the virtual environment:"
echo "  source venv/bin/activate"
echo "Then run:"
echo "  memon <process_name>"
echo "To deactivate the virtual environment, run:"
echo "  deactivate"