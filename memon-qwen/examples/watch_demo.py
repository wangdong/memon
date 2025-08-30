#!/usr/bin/env python3
"""
Demo script for watch mode
"""

import subprocess
import sys
import time


def main():
    print("Starting memon in watch mode for 10 seconds...")
    print("Press Ctrl+C to exit early")
    
    try:
        # Run memon in watch mode for 10 seconds
        process = subprocess.Popen([
            sys.executable, "-m", "memon.main", 
            "python", "-t", "2"
        ])
        
        # Let it run for 10 seconds
        time.sleep(10)
        
        # Terminate the process
        process.terminate()
        process.wait()
        
    except KeyboardInterrupt:
        print("\nDemo interrupted by user")
        if 'process' in locals():
            process.terminate()
            process.wait()


if __name__ == "__main__":
    main()