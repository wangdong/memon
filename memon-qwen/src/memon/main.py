"""
Memory Monitor - Process Tree Memory Analyzer
Analyzes memory usage of a process and its children, displaying as a tree structure
"""

import argparse
import sys
import time
from typing import Dict, List, Optional, Tuple

import psutil
import rich
from rich.console import Console
from rich.table import Table
from rich.tree import Tree
from rich.panel import Panel
from rich.layout import Layout
from rich.live import Live
from rich.text import Text


class ProcessInfo:
    """Container for process information"""
    
    def __init__(self, pid: int, name: str, rss: int, vsz: int, parent_pid: Optional[int] = None):
        self.pid = pid
        self.name = name
        self.rss = rss  # Resident Set Size in bytes
        self.vsz = vsz  # Virtual Memory Size in bytes
        self.parent_pid = parent_pid
        self.children: List['ProcessInfo'] = []


class MemoryMonitor:
    """Main memory monitoring class"""
    
    def __init__(self, no_color: bool = False):
        self.console = Console(color_system=None if no_color else "auto")
        self.no_color = no_color
        self.processes: Dict[int, ProcessInfo] = {}
        
    def _get_process_info(self, process: psutil.Process) -> ProcessInfo:
        """Extract information from a psutil.Process object"""
        try:
            pid = process.pid
            name = process.name()
            memory_info = process.memory_info()
            rss = memory_info.rss
            vsz = memory_info.vms
            parent_pid = process.ppid() if process.ppid() else None
            
            return ProcessInfo(pid, name, rss, vsz, parent_pid)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            # Return a minimal ProcessInfo object if we can't access the process
            return ProcessInfo(process.pid, "Unknown", 0, 0, None)
    
    def _build_process_tree(self, root_pid: int) -> Optional[ProcessInfo]:
        """Build process tree starting from root PID"""
        if root_pid not in self.processes:
            return None
            
        # Clear existing children relationships to avoid duplicates
        for pid in self.processes:
            self.processes[pid].children = []
            
        # Build parent-child relationships
        for pid, proc_info in self.processes.items():
            if proc_info.parent_pid in self.processes:
                self.processes[proc_info.parent_pid].children.append(proc_info)
        
        return self.processes[root_pid]
    
    def _format_memory(self, bytes_value: int) -> str:
        """Convert bytes to human readable format (MB/GB)"""
        if bytes_value == 0:
            return "0B"
            
        mb = bytes_value / (1024 * 1024)
        gb = mb / 1024
        
        if gb >= 1:
            return f"{gb:.1f}GB"
        else:
            return f"{mb:.1f}MB"
    
    def _get_memory_color(self, bytes_value: int) -> str:
        """Get color based on memory usage level"""
        if self.no_color:
            return ""
            
        mb = bytes_value / (1024 * 1024)
        
        if mb < 10:
            return "green"
        elif mb < 100:
            return "yellow"
        elif mb < 500:
            return "magenta"
        else:
            return "red"
    
    def _add_process_to_tree(self, tree: Tree, process: ProcessInfo, is_last: bool = False) -> None:
        """Add a process to the Rich tree visualization"""
        # Format memory with color
        memory_str = self._format_memory(process.rss)
        if not self.no_color:
            memory_color = self._get_memory_color(process.rss)
            memory_text = f"[{memory_color}]{memory_str}[/{memory_color}]"
        else:
            memory_text = memory_str
            
        # Create label for this process
        label = f"[bold blue][{process.pid}][/bold blue] {process.name} (RSS: {memory_text})"
        
        # Add this process to the tree
        branch = tree.add(label)
        
        # Add children
        for i, child in enumerate(process.children):
            self._add_process_to_tree(branch, child, i == len(process.children) - 1)
    
    def _find_processes_by_name(self, process_name: str) -> List[int]:
        """Find all PIDs matching the given process name"""
        matching_pids = []
        
        # Iterate through all processes
        for proc in psutil.process_iter(['pid', 'name']):
            try:
                # Check if process name matches (case insensitive)
                if process_name.lower() in proc.info['name'].lower():
                    matching_pids.append(proc.info['pid'])
            except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                # Skip processes we can't access
                continue
                
        return matching_pids
    
    def _find_root_processes(self, matching_pids: List[int]) -> List[int]:
        """Find root processes (processes whose parent is not in the matching list)"""
        root_pids = []
        
        for pid in matching_pids:
            try:
                process = psutil.Process(pid)
                parent_pid = process.ppid()
                
                # If parent is not in matching list or parent is 1 (init), consider it a root
                if parent_pid not in matching_pids or parent_pid == 1:
                    root_pids.append(pid)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                # Skip processes we can't access
                continue
                
        return root_pids
    
    def _collect_all_processes(self, root_pid: int) -> None:
        """Collect all processes in the tree starting from root_pid"""
        try:
            # Create a process object for the root
            root_process = psutil.Process(root_pid)
            
            # Add root process
            root_info = self._get_process_info(root_process)
            self.processes[root_pid] = root_info
            
            # Add all children recursively
            self._collect_children(root_process)
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
    
    def _collect_children(self, parent_process: psutil.Process) -> None:
        """Recursively collect all child processes"""
        try:
            children = parent_process.children(recursive=True)
            for child in children:
                child_info = self._get_process_info(child)
                self.processes[child.pid] = child_info
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
    
    def _calculate_total_memory(self, root: ProcessInfo) -> int:
        """Calculate total RSS memory for a process tree"""
        total_memory = root.rss  # Root's memory
        for child in root.children:
            total_memory += self._calculate_total_memory(child)
        return total_memory
    
    def _count_processes(self, root: ProcessInfo) -> int:
        """Count total number of processes in tree"""
        count = 1  # Root itself
        for child in root.children:
            count += self._count_processes(child)
        return count
    
    def analyze_process_tree(self, process_name: str) -> bool:
        """Main analysis function"""
        # Find matching processes
        matching_pids = self._find_processes_by_name(process_name)
        
        if not matching_pids:
            self.console.print(f"[red bold]❌ No processes found matching '{process_name}'[/red bold]")
            return False
        
        self.console.print(f"[green bold]✅ Found {len(matching_pids)} matching process(es)[/green bold]")
        
        # Find root processes
        root_pids = self._find_root_processes(matching_pids)
        
        if not root_pids:
            self.console.print(f"[red bold]❌ No root processes found[/red bold]")
            return False
            
        self.console.print(f"[green bold]🌳 Found {len(root_pids)} root process tree(s)[/green bold]")
        
        # Analyze each process tree
        for i, root_pid in enumerate(root_pids):
            if i > 0:
                self.console.print("[yellow bold]" + "="*60 + "[/yellow bold]")
            
            self.console.print(f"\n[cyan bold]📊 Process Tree {i+1} (Root PID: {root_pid})[/cyan bold]")
            self.console.print("[yellow bold]" + "-" * 60 + "[/yellow bold]")
            
            # Collect all processes in this tree
            self.processes = {}
            self._collect_all_processes(root_pid)
            
            # Build and print tree
            root_process = self._build_process_tree(root_pid)
            if root_process:
                self.console.print(f"[yellow bold]🎯 Root:[/yellow bold] [{root_pid}] {root_process.name}")
                self.console.print("[cyan bold]📋 Process Tree:[/cyan bold]")
                
                # Create Rich tree
                tree = Tree("")
                self._add_process_to_tree(tree, root_process)
                self.console.print(tree)
                
                # Print summary
                process_count = self._count_processes(root_process)
                total_memory = self._calculate_total_memory(root_process)
                avg_memory = total_memory // process_count if process_count > 0 else 0
                
                # Create summary table
                table = Table(show_header=False, box=None)
                table.add_column("Key", style="bright_black bold")
                table.add_column("Value")
                
                table.add_row("Tree Procs:", str(process_count))
                table.add_row("Avg Memory:", self._format_memory(avg_memory))
                table.add_row("Total Memory:", self._format_memory(total_memory))
                
                self.console.print("\n[cyan bold]📈 Summary:[/cyan bold]")
                self.console.print(table)
            else:
                self.console.print(f"[red bold]❌ Could not build process tree for PID {root_pid}[/red bold]")
        
        return True


def main():
    parser = argparse.ArgumentParser(description="Analyze memory usage of a process and its children")
    parser.add_argument("process_name", help="Name of the process to analyze")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")
    parser.add_argument("--no-color", action="store_true", help="Disable colored output")
    parser.add_argument("-t", "--watch", type=int, metavar="SECONDS", help="Watch mode - continuously update every N seconds")
    
    args = parser.parse_args()
    
    # Create memory monitor and analyze
    monitor = MemoryMonitor(no_color=args.no_color)
    
    try:
        if args.watch:
            # Watch mode - continuously update
            if args.watch <= 0:
                monitor.console.print("[red]Watch interval must be greater than 0[/red]")
                sys.exit(1)
            
            monitor.console.print(f"[cyan bold]🕒 Starting watch mode - updating every {args.watch} seconds (Press Ctrl+C to stop)[/cyan bold]")
            time.sleep(2)  # Give user time to read the message
            
            with Live(auto_refresh=False) as live:
                while True:
                    # Create a new console for each update to clear the screen
                    console = Console()
                    
                    # Add timestamp
                    timestamp = time.strftime('%Y-%m-%d %H:%M:%S')
                    console.print(f"[cyan bold]🕒 {timestamp}[/cyan bold]")
                    console.print(f"[yellow bold]🔄 Updating every {args.watch} seconds[/yellow bold]")
                    console.print("[yellow bold]" + "=" * 60 + "[/yellow bold]")
                    
                    # Run analysis
                    monitor = MemoryMonitor(no_color=args.no_color)
                    success = monitor.analyze_process_tree(args.process_name)
                    
                    # Update live display
                    live.update(console.export_text(), refresh=True)
                    
                    # Wait for next iteration
                    time.sleep(args.watch)
        else:
            # Single run mode
            success = monitor.analyze_process_tree(args.process_name)
            if not success:
                sys.exit(1)
    except KeyboardInterrupt:
        monitor.console.print("\n[red]🛑 Analysis interrupted by user[/red]")
        sys.exit(1)
    except Exception as e:
        monitor.console.print(f"[red]❌ Error: {e}[/red]")
        sys.exit(1)


if __name__ == "__main__":
    main()