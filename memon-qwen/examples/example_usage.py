#!/usr/bin/env python3
"""
Example usage of memon
"""

import subprocess
import time
import os
import signal
import sys


def main():
    # Example 1: Basic usage
    print("=== Example 1: Basic usage ===")
    print("Running: memon python")
    result = subprocess.run([sys.executable, "-m", "memon.main", "python"], 
                          capture_output=True, text=True)
    print("STDOUT:")
    print(result.stdout)
    if result.stderr:
        print("STDERR:")
        print(result.stderr)
    print("=" * 50 + "\n")
    
    # Example 2: No color output
    print("=== Example 2: No color output ===")
    print("Running: memon python --no-color")
    result = subprocess.run([sys.executable, "-m", "memon.main", "python", "--no-color"], 
                          capture_output=True, text=True)
    print("STDOUT:")
    print(result.stdout)
    if result.stderr:
        print("STDERR:")
        print(result.stderr)
    print("=" * 50 + "\n")


if __name__ == "__main__":
    main()
