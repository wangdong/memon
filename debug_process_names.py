#!/usr/bin/env python3
"""
Debug script to check what process names are actually being captured
"""

import subprocess
import platform

def get_all_processes():
    """Get all processes and print their names for debugging"""
    system = platform.system().lower()
    
    if system == "darwin":  # macOS
        # Use ps with -c flag to get just the executable name
        # Use wider columns to avoid truncation
        output = subprocess.run("ps -c -eo pid,ppid,comm,rss,vsz | head -20", shell=True, capture_output=True, text=True).stdout
        print("Sample process output structure:")
        print(output)
        print("=" * 60)
        
        output = subprocess.run("ps -c -eo pid,ppid,comm,rss,vsz", shell=True, capture_output=True, text=True).stdout
        lines = output.split('\n')[1:]  # Skip header
        
        print("=== macOS Process Names (using ps -c) ===")
        processes = []
        for line in lines:
            if line.strip():
                # Handle case where comm might contain spaces due to truncation
                # We know the format: PID PPID COMM RSS VSZ
                # RSS and VSZ are always numbers
                parts = line.split()
                if len(parts) >= 5:
                    try:
                        pid = int(parts[0])
                        ppid = int(parts[1])
                        
                        # Find RSS and VSZ (last two numeric fields)
                        vsz = int(parts[-1])
                        rss = int(parts[-2])
                        
                        # Everything between PPID and RSS is the command name
                        comm = ' '.join(parts[2:-2])
                        
                        processes.append({
                            'pid': pid, 
                            'ppid': ppid, 
                            'name': comm, 
                            'rss': rss, 
                            'vsz': vsz
                        })
                        
                        # Print all process names for debugging
                        print(f"PID: {pid:6d} PPID: {ppid:6d} Name: {comm:30} RSS: {rss:8d} KB")
                    except (ValueError, IndexError):
                        # Skip malformed lines
                        continue
        
        return processes
    else:
        print("This debug script is designed for macOS")
        return []

def find_feishu_processes():
    """Look for Feishu-related processes"""
    processes = get_all_processes()
    
    print("\n=== Looking for Feishu/Lark processes ===")
    
    feishu_processes = []
    lark_processes = []
    
    for proc in processes:
        name = proc['name'].lower()
        if 'feishu' in name:
            feishu_processes.append(proc)
            print(f"✅ Feishu process found: PID {proc['pid']:6d} Name: '{proc['name']}'")
        elif 'lark' in name:
            lark_processes.append(proc)
            print(f"✅ Lark process found: PID {proc['pid']:6d} Name: '{proc['name']}'")
    
    print(f"\nTotal Feishu processes: {len(feishu_processes)}")
    print(f"Total Lark processes: {len(lark_processes)}")
    
    return feishu_processes, lark_processes

def check_parent_child_relationships():
    """Check parent-child relationships for Feishu/Lark processes"""
    processes = get_all_processes()
    
    print("\n=== Checking Parent-Child Relationships ===")
    
    # Create a dictionary for quick lookup
    proc_dict = {p['pid']: p for p in processes}
    
    for proc in processes:
        name = proc['name'].lower()
        if 'feishu' in name or 'lark' in name:
            parent_pid = proc['ppid']
            parent = proc_dict.get(parent_pid)
            
            print(f"\nProcess: {proc['name']} (PID: {proc['pid']})")
            print(f"  Parent PID: {parent_pid}")
            
            if parent:
                print(f"  Parent Name: {parent['name']}")
            else:
                print(f"  Parent Name: NOT FOUND")
            
            # Find children
            children = [p for p in processes if p['ppid'] == proc['pid']]
            print(f"  Children ({len(children)}):")
            for child in children:
                print(f"    - {child['name']} (PID: {child['pid']})")

if __name__ == "__main__":
    print("🔍 Debugging Process Name Detection\n")
    
    # Get all processes first
    processes = get_all_processes()
    print(f"\nTotal processes found: {len(processes)}")
    
    # Look for Feishu/Lark specifically
    find_feishu_processes()
    
    # Check relationships
    check_parent_child_relationships()