#!/usr/bin/env python3
"""
Memory Monitor - Process Tree Memory Analyzer
Analyzes memory usage of a process and its children, displaying as a tree structure
"""

import subprocess
import argparse
import sys
import re
import platform
import os
from typing import Dict, List, Tuple, Optional


class Colors:
    """ANSI color codes for cross-platform colored output"""
    
    # Reset
    RESET = '\033[0m'
    
    # Foreground colors
    BLACK = '\033[30m'
    RED = '\033[31m'
    GREEN = '\033[32m'
    YELLOW = '\033[33m'
    BLUE = '\033[34m'
    MAGENTA = '\033[35m'
    CYAN = '\033[36m'
    WHITE = '\033[37m'
    
    # Bright foreground colors
    BRIGHT_BLACK = '\033[90m'
    BRIGHT_RED = '\033[91m'
    BRIGHT_GREEN = '\033[92m'
    BRIGHT_YELLOW = '\033[93m'
    BRIGHT_BLUE = '\033[94m'
    BRIGHT_MAGENTA = '\033[95m'
    BRIGHT_CYAN = '\033[96m'
    BRIGHT_WHITE = '\033[97m'
    
    # Background colors
    BG_BLACK = '\033[40m'
    BG_RED = '\033[41m'
    BG_GREEN = '\033[42m'
    BG_YELLOW = '\033[43m'
    BG_BLUE = '\033[44m'
    BG_MAGENTA = '\033[45m'
    BG_CYAN = '\033[46m'
    BG_WHITE = '\033[47m'
    
    # Styles
    BOLD = '\033[1m'
    DIM = '\033[2m'
    UNDERLINE = '\033[4m'
    BLINK = '\033[5m'
    REVERSE = '\033[7m'
    HIDDEN = '\033[8m'
    
    @classmethod
    def should_use_colors(cls):
        """Determine if colors should be used based on environment"""
        # Check if NO_COLOR environment variable is set
        if os.getenv('NO_COLOR'):
            return False
        
        # Check if output is a terminal
        if not sys.stdout.isatty():
            return False
        
        # Windows support
        if platform.system() == 'Windows':
            try:
                import ctypes
                kernel32 = ctypes.windll.kernel32
                # Enable ANSI colors on Windows 10+
                kernel32.SetConsoleMode(kernel32.GetStdHandle(-11), 7)
                return True
            except:
                return False
        
        # Unix-like systems
        return True


class ProcessInfo:
    def __init__(self, pid: int, name: str, rss: int, vsz: int, parent_pid: Optional[int] = None):
        self.pid = pid
        self.name = name
        self.rss = rss  # Resident Set Size in bytes
        self.vsz = vsz  # Virtual Memory Size in bytes
        self.parent_pid = parent_pid
        self.children = []
        self.is_max_memory = False  # Flag to mark if this process has max memory
        self.is_second_max_memory = False  # Flag to mark if this process has second max memory
        self.is_third_max_memory = False  # Flag to mark if this process has third max memory

    def add_child(self, child: 'ProcessInfo'):
        self.children.append(child)



class MemoryMonitor:
    def __init__(self, no_color=False):
        self.processes: Dict[int, ProcessInfo] = {}
        self.system = platform.system().lower()
        self.no_color = no_color or not Colors.should_use_colors()
        self.max_rss = 0  # Track maximum RSS across all processes
        self.second_max_rss = 0  # Track second maximum RSS across all processes
        
    def _execute_command(self, command: str) -> str:
        """Execute a shell command and return output"""
        try:
            result = subprocess.run(command, shell=True, capture_output=True, text=True)
            return result.stdout.strip()
        except Exception as e:
            print(f"Error executing command '{command}': {e}")
            return ""
    
    def _get_all_processes(self) -> List[Dict]:
        """Get all processes using system commands"""
        processes = []
        
        if self.system == "darwin":  # macOS
            # Use ps with -c flag to get just the executable name (not full path)
            output = self._execute_command("ps -c -eo pid,ppid,comm,rss,vsz")
            lines = output.split('\n')[1:]  # Skip header
            for line in lines:
                if line.strip():
                    # Handle case where comm might contain spaces
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
                                'rss': rss * 1024,  # Convert from KB to bytes
                                'vsz': vsz * 1024   # Convert from KB to bytes
                            })
                        except (ValueError, IndexError):
                            continue
        else:  # Linux and other Unix-like systems
            output = self._execute_command("ps -eo pid,ppid,command,rss,vsz")
            lines = output.split('\n')[1:]  # Skip header
            for line in lines:
                if line.strip():
                    # Handle case where command might contain spaces
                    parts = line.split()
                    if len(parts) >= 5:
                        try:
                            pid = int(parts[0])
                            ppid = int(parts[1])
                            # The command is everything between ppid and rss/vsz
                            # Find where rss starts (it should be a number)
                            numeric_indices = []
                            for i in range(2, len(parts)):
                                if parts[i].isdigit():
                                    numeric_indices.append(i)
                            
                            if len(numeric_indices) >= 2:
                                rss_index = numeric_indices[-2]
                                vsz_index = numeric_indices[-1]
                            
                            if rss_index != -1 and vsz_index < len(parts):
                                command = ' '.join(parts[2:rss_index])
                                rss = int(parts[rss_index]) * 1024  # Convert from KB to bytes
                                vsz = int(parts[vsz_index]) * 1024  # Convert from KB to bytes
                                
                                # Extract clean process name
                                clean_name = self._extract_process_name(command)
                                
                                processes.append({
                                    'pid': pid, 
                                    'ppid': ppid, 
                                    'name': clean_name, 
                                    'rss': rss, 
                                    'vsz': vsz
                                })
                        except (ValueError, IndexError):
                            continue
        
        return processes
    
    def _extract_process_name(self, command: str) -> str:
        """Extract process name from full command path"""
        # Remove path and get just the executable name
        if '/' in command:
            name = command.split('/')[-1]
            # Remove any arguments after the executable
            if ' ' in name:
                name = name.split(' ')[0]
        else:
            # If no path, still remove arguments
            name = command.split(' ')[0]
        
        return name
    
    def get_processes_by_name(self, process_name: str) -> List[int]:
        """Find all PIDs matching the given process name"""
        matching_pids = []
        
        try:
            # Get all processes
            processes = self._get_all_processes()
            
            for proc in processes:
                self._add_process_info(proc)
                # More precise matching for process names
                proc_name = proc['name'].lower()
                target_name = process_name.lower()
                
                # Check for exact match or specific app name variations
                if (self._is_exact_match(proc_name, target_name) or 
                    self._is_app_name_match(proc_name, target_name)):
                    matching_pids.append(proc['pid'])
                    
        except Exception as e:
            print(f"Error getting processes: {e}")
            
        return matching_pids
    
    def _is_exact_match(self, proc_name: str, target_name: str) -> bool:
        """Check if process name exactly matches target name"""
        # Handle truncated process names (common on macOS with ps -c)
        # If target name is being searched and process name might be truncated
        if len(proc_name) >= 15 and target_name.startswith(proc_name):
            return True
        
        # Handle case where target name is long and might be truncated
        if len(target_name) > 15 and proc_name.startswith(target_name[:15]):
            return True
        
        # Exact match
        if proc_name == target_name:
            return True
        
        # Check if target name starts with process name (for truncated names)
        if target_name.startswith(proc_name):
            return True
            
        # Check if process name starts with target name (for partial matching)
        if proc_name.startswith(target_name):
            return True
        
        # Extract basename from process name if it contains a path
        proc_basename = proc_name
        if '/' in proc_name:
            proc_basename = proc_name.split('/')[-1]
        
        # Extract basename from target name if it contains a path
        target_basename = target_name
        if '/' in target_name:
            target_basename = target_name.split('/')[-1]
        
        # Handle common executable extensions
        base_proc = proc_basename
        if proc_basename.endswith(('.exe', '.app', '.bin', '.run')):
            base_proc = proc_basename[:-4]
        
        base_target = target_basename
        if target_basename.endswith(('.exe', '.app', '.bin', '.run')):
            base_target = target_basename[:-4]
            
        return base_proc == base_target
    
    def _is_app_name_match(self, proc_name: str, target_name: str) -> bool:
        """Check if process name matches app name in different formats"""
        # Only handle .app extension for macOS applications
        if target_name.endswith('.app'):
            app_name = target_name[:-4]  # Remove .app suffix
            # Check if process name matches the app name (without .app)
            if proc_name == app_name.lower():
                return True
        
        # Check for common macOS app naming patterns
        # Some apps have process names like "App Name" when app is "AppName"
        if ' ' in target_name:
            compact_name = target_name.replace(' ', '').lower()
            if proc_name == compact_name:
                return True
        
        return False
    
    def _add_process_info(self, proc_info: Dict):
        """Add process info to our internal dictionary"""
        try:
            pid = proc_info['pid']
            name = proc_info['name']
            rss = proc_info['rss']
            vsz = proc_info['vsz']
            parent_pid = proc_info['ppid']
            
            self.processes[pid] = ProcessInfo(pid, name, rss, vsz, parent_pid)
        except KeyError:
            pass
    
    def build_process_tree(self, root_pid: int) -> Optional[ProcessInfo]:
        """Build process tree starting from root PID"""
        if root_pid not in self.processes:
            return None
            
        # Clear existing children relationships to avoid duplicates
        for pid in self.processes:
            self.processes[pid].children = []
            
        # Get all processes and their children
        for pid, proc_info in self.processes.items():
            if proc_info.parent_pid in self.processes:
                self.processes[proc_info.parent_pid].add_child(proc_info)
        
        return self.processes[root_pid]
    
    def find_root_processes(self, matching_pids: List[int]) -> List[int]:
        """Find root processes (processes whose parent is not in the matching list)"""
        root_pids = []
        
        for pid in matching_pids:
            if pid in self.processes:
                proc_info = self.processes[pid]
                # If parent is not in matching list or parent is 1 (launchd), consider it a root
                if (proc_info.parent_pid not in matching_pids or 
                    proc_info.parent_pid == 1 or 
                    proc_info.parent_pid not in self.processes):
                    root_pids.append(pid)
                    
        return root_pids
    
    def format_memory(self, bytes_value: int) -> str:
        """Convert bytes to human readable format (MB/GB)"""
        if bytes_value == 0:
            return "0B"
        
        mb = bytes_value / (1024 * 1024)
        gb = mb / 1024
        
        if gb >= 1:
            return f"{gb:.1f}GB"
        else:
            return f"{mb:.1f}MB"
    
    def get_memory_color(self, bytes_value: int, is_max_memory: bool = False, is_second_max_memory: bool = False, is_third_max_memory: bool = False) -> str:
        """Get color based on memory usage level"""
        if self.no_color:
            return ""
        
        # If this is the process with maximum memory, use red background with white foreground
        if is_max_memory:
            return Colors.BG_RED + Colors.WHITE + Colors.BOLD
        
        # If this is the process with second maximum memory, use pink background with white foreground
        if is_second_max_memory:
            return Colors.BG_MAGENTA + Colors.WHITE + Colors.BOLD
        
        # If this is the process with third maximum memory, use cyan background with black foreground
        if is_third_max_memory:
            return Colors.BG_CYAN + Colors.BLACK + Colors.BOLD
        
        mb = bytes_value / (1024 * 1024)
        
        if mb < 10:
            return Colors.GREEN
        elif mb < 100:
            return Colors.YELLOW
        elif mb < 500:
            return Colors.MAGENTA
        else:
            return Colors.RED
    
    def get_colored_memory_str(self, bytes_value: int, is_max_memory: bool = False, is_second_max_memory: bool = False, is_third_max_memory: bool = False) -> str:
        """Get memory string with color coding"""
        color = self.get_memory_color(bytes_value, is_max_memory, is_second_max_memory, is_third_max_memory)
        memory_str = self.format_memory(bytes_value)
        if self.no_color:
            return memory_str
        return f"{color}{memory_str}{Colors.RESET}"
    
    def print_tree(self, root: ProcessInfo, level: int = 0, is_last: bool = False, total_memory: int = 0):
        """Print process tree with memory information"""
        if root is None:
            return
            
        # Format the current node with colors
        memory_str = self.get_colored_memory_str(root.rss, root.is_max_memory, root.is_second_max_memory, root.is_third_max_memory)
        
        # Calculate and format overall percentage if total_memory is provided
        percentage_str = ""
        if total_memory > 0:
            percentage = (root.rss / total_memory) * 100
            percentage_str = f" ({percentage:.1f}%)"
        
        # Create colored tree structure
        tree_prefix = ""
        if level > 0:
            if not self.no_color:
                tree_prefix = Colors.CYAN + "│   " * (level - 1) + Colors.RESET
                tree_prefix += Colors.CYAN + ("└── " if is_last else "├── ") + Colors.RESET
            else:
                tree_prefix = "│   " * (level - 1) + ("└── " if is_last else "├── ")
        
        # Color PID based on level
        if not self.no_color:
            pid_color = Colors.BRIGHT_BLACK + Colors.BOLD if level == 0 else Colors.BRIGHT_BLUE + Colors.BOLD
            name_color = Colors.BLUE + Colors.BOLD
            rss_label = Colors.BOLD + "RSS:" + Colors.RESET
        else:
            pid_color = ""
            name_color = ""
            rss_label = "RSS:"
        
        # Add emoji for memory ranking
        rank_emoji = ""
        if root.is_max_memory:
            rank_emoji = "🥇"
        elif root.is_second_max_memory:
            rank_emoji = "🥈"
        elif root.is_third_max_memory:
            rank_emoji = "🥉"
        
        # Print current process with colors and ranking emoji
        print(f"{tree_prefix}{pid_color}[{root.pid}]{Colors.RESET if not self.no_color else ''} {name_color}{root.name}{Colors.RESET if not self.no_color else ''} ({rss_label} {memory_str}{rank_emoji}{percentage_str})")
        
        # Print children
        for i, child in enumerate(root.children):
            self.print_tree(child, level + 1, i == len(root.children) - 1, total_memory)
    
    def analyze_process_tree(self, process_name: str) -> bool:
        """Main analysis function"""
        if self.no_color:
            print(f"🔍 Searching for processes matching: {process_name}")
        else:
            print(f"{Colors.YELLOW}{Colors.BOLD}🔍 Searching for processes matching:{Colors.RESET} {Colors.CYAN}{process_name}{Colors.RESET}")
        
        # Find matching processes
        matching_pids = self.get_processes_by_name(process_name)
        
        # Reset memory flags for all processes
        for pid in self.processes:
            self.processes[pid].is_max_memory = False
            self.processes[pid].is_second_max_memory = False
            self.processes[pid].is_third_max_memory = False
        
        if not matching_pids:
            if self.no_color:
                print(f"❌ No processes found matching '{process_name}'")
            else:
                print(f"{Colors.RED}{Colors.BOLD}❌ No processes found matching '{process_name}'{Colors.RESET}")
            return False
        
        if self.no_color:
            print(f"✅ Found {len(matching_pids)} matching process(es)")
        else:
            print(f"{Colors.GREEN}{Colors.BOLD}✅ Found {len(matching_pids)} matching process(es){Colors.RESET}")
        
        # Find root processes
        root_pids = self.find_root_processes(matching_pids)
        
        if not root_pids:
            if self.no_color:
                print(f"❌ No root processes found")
            else:
                print(f"{Colors.RED}{Colors.BOLD}❌ No root processes found{Colors.RESET}")
            return False
            
        if self.no_color:
            print(f"🌳 Found {len(root_pids)} root process tree(s)")
        else:
            print(f"{Colors.GREEN}{Colors.BOLD}🌳 Found {len(root_pids)} root process tree(s){Colors.RESET}")
        
        
        # Analyze each process tree
        for i, root_pid in enumerate(root_pids):
            if i > 0:
                if self.no_color:
                    print(f"\n{'='*60}")
                else:
                    print(f"\n{Colors.YELLOW}{Colors.BOLD}{'='*60}{Colors.RESET}")
            
            if self.no_color:
                print(f"\n📊 Process Tree {i+1} (Root PID: {root_pid})")
                print(f"{'-' * 60}")
            else:
                print(f"\n{Colors.CYAN}{Colors.BOLD}📊 Process Tree {i+1} (Root PID: {root_pid}){Colors.RESET}")
                print(f"{Colors.YELLOW}{Colors.BOLD}{'-' * 60}{Colors.RESET}")
            
            # Build and print tree
            root_process = self.build_process_tree(root_pid)
            if root_process:
                if self.no_color:
                    print(f"🎯 Root: [{root_pid}] {root_process.name}")
                    print(f"📋 Process Tree:")
                else:
                    print(f"{Colors.YELLOW}{Colors.BOLD}🎯 Root:{Colors.RESET} [{root_pid}] {root_process.name}")
                    print(f"{Colors.CYAN}{Colors.BOLD}📋 Process Tree:{Colors.RESET}")
                
                # Collect all RSS values in this tree and find max, second max, and third max
                all_rss_in_tree = self._collect_all_rss_in_tree(root_process)
                
                # Print summary
                process_count = self._count_processes(root_process)
                total_memory = self._calculate_total_memory(root_process)
                
                # Mark processes with max, second max, and third max memory
                if len(all_rss_in_tree) > 0:
                    tree_max_rss = max(all_rss_in_tree)
                    filtered_rss = [rss for rss in all_rss_in_tree if rss != tree_max_rss]
                    tree_second_max_rss = max(filtered_rss) if filtered_rss else 0
                    
                    # Find third max
                    if len(filtered_rss) > 0:
                        third_filtered_rss = [rss for rss in filtered_rss if rss != tree_second_max_rss]
                        tree_third_max_rss = max(third_filtered_rss) if third_filtered_rss else 0
                    else:
                        tree_third_max_rss = 0
                    
                    # Mark processes with max, second max, and third max memory
                    self._mark_memory_highlights_in_tree(root_process, tree_max_rss, tree_second_max_rss, tree_third_max_rss)
                
                self.print_tree(root_process, total_memory=total_memory)
                top_processes = self._collect_top_memory_processes_with_percentages(root_process, total_memory)
                
                if self.no_color:
                    print(f"\n📈 Summary:")
                    print(f"   Tree Processes: {process_count}")
                    print(f"   Total Memory: {self.format_memory(total_memory)}")
                    
                    # Calculate and print combined percentage of top 3
                    if top_processes:
                        combined_percentage = sum(percentage for _, percentage in top_processes)
                        combined_memory = sum(process.rss for process, _ in top_processes)
                        print(f"   Top 3 Combined: {self.format_memory(combined_memory)} ({combined_percentage:.1f}%)")
                else:
                    print(f"\n{Colors.CYAN}{Colors.BOLD}📈 Summary:{Colors.RESET}")
                    print(f"   {Colors.BRIGHT_BLACK}{Colors.BOLD}Tree Procs:{Colors.RESET} {process_count}")
                    print(f"   {Colors.BRIGHT_BLACK}{Colors.BOLD}Total Memory:{Colors.RESET} {self.get_colored_memory_str(total_memory)}")
                    
                    # Calculate and print combined percentage of top 3
                    if top_processes:
                        combined_percentage = sum(percentage for _, percentage in top_processes)
                        combined_memory = sum(process.rss for process, _ in top_processes)
                        print(f"   {Colors.BRIGHT_BLACK}{Colors.BOLD}Top 3 Combined:{Colors.RESET} {self.get_colored_memory_str(combined_memory)} ({combined_percentage:.1f}%)")
            else:
                if self.no_color:
                    print(f"❌ Could not build process tree for PID {root_pid}")
                else:
                    print(f"{Colors.RED}{Colors.BOLD}❌ Could not build process tree for PID {root_pid}{Colors.RESET}")
        
        return True
    
    def _count_processes(self, root: ProcessInfo) -> int:
        """Count total number of processes in tree"""
        count = 1  # Root itself
        for child in root.children:
            count += self._count_processes(child)
        return count
    
    def _calculate_total_memory(self, root: ProcessInfo) -> int:
        """Calculate total RSS memory for a process tree"""
        total_memory = root.rss  # Root's memory
        for child in root.children:
            total_memory += self._calculate_total_memory(child)
        return total_memory
    
    def _collect_all_rss_in_tree(self, root: ProcessInfo) -> List[int]:
        """Collect all RSS values from processes in the tree"""
        rss_values = [root.rss]  # Root's RSS
        for child in root.children:
            rss_values.extend(self._collect_all_rss_in_tree(child))
        return rss_values
    
    def _collect_top_memory_processes_with_percentages(self, root: ProcessInfo, total_memory: int) -> List[Tuple[ProcessInfo, float]]:
        """Collect top three memory processes with their percentages"""
        all_processes = []
        self._collect_all_processes_in_tree(root, all_processes)
        
        # Sort by RSS in descending order
        all_processes.sort(key=lambda p: p.rss, reverse=True)
        
        # Get top three processes with percentages
        top_processes = []
        for i, process in enumerate(all_processes[:3]):
            if total_memory > 0:
                percentage = (process.rss / total_memory) * 100
            else:
                percentage = 0.0
            top_processes.append((process, percentage))
        
        return top_processes
    
    def _collect_all_processes_in_tree(self, root: ProcessInfo, processes: List[ProcessInfo]):
        """Collect all processes in the tree"""
        processes.append(root)
        for child in root.children:
            self._collect_all_processes_in_tree(child, processes)
    
    def _mark_memory_highlights_in_tree(self, root: ProcessInfo, max_rss: int, second_max_rss: int, third_max_rss: int = 0):
        """Mark processes with max, second max, and third max memory in the tree"""
        if root.rss == max_rss:
            root.is_max_memory = True
        elif root.rss == second_max_rss and second_max_rss > 0:
            root.is_second_max_memory = True
        elif root.rss == third_max_rss and third_max_rss > 0:
            root.is_third_max_memory = True
        
        for child in root.children:
            self._mark_memory_highlights_in_tree(child, max_rss, second_max_rss, third_max_rss)


def main():
    parser = argparse.ArgumentParser(description="Analyze memory usage of a process and its children")
    parser.add_argument("process_name", help="Name of the process to analyze")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")
    parser.add_argument("--no-color", action="store_true", help="Disable colored output")
    
    args = parser.parse_args()
    
    # Create memory monitor and analyze
    monitor = MemoryMonitor(no_color=args.no_color)
    
    try:
        success = monitor.analyze_process_tree(args.process_name)
        if not success:
            sys.exit(1)
    except KeyboardInterrupt:
        if monitor.no_color:
            print("\n🛑 Analysis interrupted by user")
        else:
            print(f"\n{Colors.RED}🛑 Analysis interrupted by user{Colors.RESET}")
        sys.exit(1)
    except Exception as e:
        if monitor.no_color:
            print(f"❌ Error: {e}")
        else:
            print(f"{Colors.RED}❌ Error: {e}{Colors.RESET}")
        sys.exit(1)


if __name__ == "__main__":
    main()