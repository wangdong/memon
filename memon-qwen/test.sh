#!/bin/bash
# Test script for memon-qwen

set -e

echo "Testing memon-qwen installation and functionality"

# Create a virtual environment
python3 -m venv venv
source venv/bin/activate

# Install the package in development mode
pip install -e ".[dev]"

# Run tests
pytest

# Run basic functionality test
echo "Testing basic functionality..."
python -c "import memon; print('memon imported successfully')"

# Deactivate virtual environment
deactivate

echo "All tests passed!"