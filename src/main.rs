// Memory Monitor - Process Tree Memory Analyzer
// Analyzes memory usage of a process and its children, displaying as a tree structure

use clap::Parser;
use std::process::Command;
use std::collections::HashMap;
use std::env;

// ANSI color codes for cross-platform colored output
mod colors {
    // Reset
    pub const RESET: &str = "\x1b[0m";
    
    // Foreground colors
    pub const WHITE: &str = "\x1b[37m";
    pub const RED: &str = "\x1b[31m";
    pub const CYAN: &str = "\x1b[36m";
    
    // Background colors
    pub const BG_RED: &str = "\x1b[41m";
    
    // Styles
    pub const BOLD: &str = "\x1b[1m";
    
    // Check if colors should be used
    pub fn should_use_colors(no_color_flag: bool) -> bool {
        // Check if NO_COLOR environment variable is set
        if no_color_flag || std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        
        // Check if output is a terminal
        // This is a simplified check. In a real application, you might want to use a crate like `atty`.
        true
    }
    
    // Functions to combine colors
    pub fn combine_colors(color1: &str, color2: &str) -> String {
        format!("{}{}", color1, color2)
    }
}

// Command line arguments
#[derive(Parser, Debug)]
#[clap(
    name = "memon",
    version = "0.1.0",
    author = "Your Name <you@example.com>",
    about = "Analyzes memory usage of a process and its children"
)]
struct Args {
    /// Name of the process to analyze
    #[clap(name = "PROCESS_NAME")]
    process_name: String,
    
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
    
    /// Disable colored output
    #[clap(long)]
    no_color: bool,
    
    /// Watch mode - continuously update every N seconds
    #[clap(short, long)]
    watch: Option<u64>,
}

// Process information structure
#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    rss: u64, // Resident Set Size in bytes
    parent_pid: Option<u32>,
    children: Vec<u32>,
    is_max_memory: bool,
    is_second_max_memory: bool,
    is_third_max_memory: bool,
}

impl ProcessInfo {
    fn new(pid: u32, name: String, rss: u64, parent_pid: Option<u32>) -> Self {
        ProcessInfo {
            pid,
            name,
            rss,
            parent_pid,
            children: Vec::new(),
            is_max_memory: false,
            is_second_max_memory: false,
            is_third_max_memory: false,
        }
    }
    
    fn add_child(&mut self, child_pid: u32) {
        self.children.push(child_pid);
    }
}

// Memory Monitor
struct MemoryMonitor {
    processes: HashMap<u32, ProcessInfo>,
    no_color: bool,
}

impl MemoryMonitor {
    fn new(no_color: bool) -> Self {
        MemoryMonitor {
            processes: HashMap::new(),
            no_color,
        }
    }
    
    // Execute a shell command and return output
    fn execute_command(&self, command: &str) -> Result<String, std::io::Error> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    // Get all processes using system commands
    fn get_all_processes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let os_type = env::consts::OS;
        let output = if os_type == "macos" {
            self.execute_command("ps -c -eo pid,ppid,comm,rss,vsz")?
        } else {
            self.execute_command("ps -eo pid,ppid,command,rss,vsz")?
        };
        
        let lines: Vec<&str> = output.lines().collect();
        // Skip header line
        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let pid = parts[0].parse::<u32>()?;
                let ppid = parts[1].parse::<u32>()?;
                let rss = parts[parts.len()-2].parse::<u64>()? * 1024; // Convert from KB to bytes
                
                // Extract process name
                let name = if os_type == "macos" {
                    parts[2].to_string()
                } else {
                    // For Linux, the command might contain spaces, so we need to handle it differently
                    // We'll take everything between ppid and rss as the command
                    let mut name_parts = Vec::new();
                    for i in 2..parts.len()-2 {
                        name_parts.push(parts[i]);
                    }
                    name_parts.join(" ")
                };
                
                self.processes.insert(pid, ProcessInfo::new(pid, name, rss, Some(ppid)));
            }
        }
        
        Ok(())
    }
    
    // Build process tree starting from root PID
    fn build_process_tree(&mut self, root_pid: u32) -> Option<ProcessInfo> {
        if !self.processes.contains_key(&root_pid) {
            return None;
        }
        
        // Clear existing children relationships to avoid duplicates
        for (_, proc_info) in self.processes.iter_mut() {
            proc_info.children.clear();
        }
        
        // Get all processes and their children
        let pids: Vec<u32> = self.processes.keys().cloned().collect();
        for pid in pids {
            if let Some(parent_pid) = self.processes[&pid].parent_pid {
                if self.processes.contains_key(&parent_pid) {
                    self.processes.get_mut(&parent_pid).unwrap().add_child(pid);
                }
            }
        }
        
        self.processes.get(&root_pid).cloned()
    }
    
    // Find root processes (processes whose parent is not in the matching list)
    fn find_root_processes(&self, matching_pids: &[u32]) -> Vec<u32> {
        let mut root_pids = Vec::new();
        
        for &pid in matching_pids {
            if let Some(proc_info) = self.processes.get(&pid) {
                // If parent is not in matching list or parent is 1 (launchd), consider it a root
                if let Some(parent_pid) = proc_info.parent_pid {
                    if !matching_pids.contains(&parent_pid) || parent_pid == 1 || !self.processes.contains_key(&parent_pid) {
                        root_pids.push(pid);
                    }
                } else {
                    root_pids.push(pid);
                }
            }
        }
        
        root_pids
    }
    
    // Convert bytes to human readable format (MB/GB)
    fn format_memory(&self, bytes_value: u64) -> String {
        if bytes_value == 0 {
            return "0B".to_string();
        }
        
        let mb = bytes_value as f64 / (1024.0 * 1024.0);
        let gb = mb / 1024.0;
        
        if gb >= 1.0 {
            format!("{:.1}GB", gb)
        } else {
            format!("{:.1}MB", mb)
        }
    }
    
    // Get color based on memory usage level
    fn get_memory_color(&self, _bytes_value: u64, is_max_memory: bool, is_second_max_memory: bool, is_third_max_memory: bool) -> String {
        if self.no_color {
            return String::new();
        }
        
        // Use red background with white foreground for top 1-3 memory processes
        if is_max_memory || is_second_max_memory || is_third_max_memory {
            return colors::combine_colors(colors::BG_RED, colors::WHITE);
        }
        
        // Default foreground color for non-trophy processes
        String::new()
    }
    
    // Get memory string with color coding
    fn get_colored_memory_str(&self, bytes_value: u64, is_max_memory: bool, is_second_max_memory: bool, is_third_max_memory: bool) -> String {
        let color = self.get_memory_color(bytes_value, is_max_memory, is_second_max_memory, is_third_max_memory);
        let memory_str = self.format_memory(bytes_value);
        if self.no_color {
            memory_str
        } else {
            format!("{}{}{}", color, memory_str, colors::RESET)
        }
    }
    
    // Print process tree with memory information
    fn print_tree(&self, root: &ProcessInfo, level: usize, is_last: bool, total_memory: u64) {
        // Format the current node with colors
        let memory_str = self.get_colored_memory_str(root.rss, root.is_max_memory, root.is_second_max_memory, root.is_third_max_memory);
        
        // Calculate and format overall percentage if total_memory is provided
        let _percentage_str = if total_memory > 0 {
            let percentage = (root.rss as f64 / total_memory as f64) * 100.0;
            format!(" ({:.1}%)", percentage)
        } else {
            String::new()
        };
        
        // Create compact tree structure
        let tree_prefix = if level > 0 {
            if !self.no_color {
                let mut prefix = String::new();
                for _ in 0..(level - 1) {
                    prefix.push_str("  ");
                }
                prefix.push_str(if is_last { "└─ " } else { "├─ " });
                prefix
            } else {
                let mut prefix = String::new();
                for _ in 0..(level - 1) {
                    prefix.push_str("  ");
                }
                prefix.push_str(if is_last { "└─ " } else { "├─ " });
                prefix
            }
        } else {
            String::new()
        };
        
        // PID coloring
        let pid_color = if !self.no_color { colors::BOLD } else { "" };
        
        // Add emoji for memory ranking
        let rank_emoji = if root.is_max_memory {
            "🥇"
        } else if root.is_second_max_memory {
            "🥈"
        } else if root.is_third_max_memory {
            "🥉"
        } else {
            ""
        };
        
        // Truncate or pad process name to 40 characters
        let display_name = if root.name.len() > 40 {
            format!("{}...", &root.name[..37])
        } else {
            format!("{:40}", root.name)
        };

        // Print process info with 40-character process name
        println!("{}{}{} {} {}{}", 
                 tree_prefix, pid_color, root.pid, display_name, memory_str, rank_emoji);
        
        // Print children
        let child_count = root.children.len();
        for (i, child_pid) in root.children.iter().enumerate() {
            if let Some(child) = self.processes.get(child_pid) {
                self.print_tree(child, level + 1, i == child_count - 1, total_memory);
            }
        }
    }
    
    // Main analysis function
    fn analyze_process_tree(&mut self, process_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let search_msg = if self.no_color {
            format!("Searching: {}", process_name)
        } else {
            format!("{}Searching:{} {}{}", 
                    colors::BOLD, colors::CYAN, process_name, colors::RESET)
        };
        println!("{}", search_msg);
        
        // Get all processes
        self.get_all_processes()?;
        
        // Find matching processes with improved matching logic
        let matching_pids: Vec<u32> = self.processes
            .iter()
            .filter(|(_, proc_info)| {
                self.is_process_matching(&proc_info.name, process_name)
            })
            .map(|(&pid, _)| pid)
            .collect();
        
        if matching_pids.is_empty() {
            let not_found_msg = if self.no_color {
                format!("No processes found matching '{}'", process_name)
            } else {
                format!("{}{}No processes found matching '{}{}'", 
                        colors::RED, colors::BOLD, process_name, colors::RESET)
            };
            println!("{}", not_found_msg);
            return Ok(false);
        }
        
        let found_msg = if self.no_color {
            format!("Found {} procs", matching_pids.len())
        } else {
            format!("{}Found {} procs{}", 
                    colors::BOLD, matching_pids.len(), colors::RESET)
        };
        println!("{}", found_msg);
        
        // Find root processes
        let root_pids = self.find_root_processes(&matching_pids);
        
        if root_pids.is_empty() {
            let no_root_msg = if self.no_color {
                "No root processes found".to_string()
            } else {
                format!("{}{}No root processes found{}", colors::RED, colors::BOLD, colors::RESET)
            };
            println!("{}", no_root_msg);
            return Ok(false);
        }
        
        let root_msg = if self.no_color {
            format!("Found {} trees", root_pids.len())
        } else {
            format!("{}Found {} trees{}", 
                    colors::BOLD, root_pids.len(), colors::RESET)
        };
        println!("{}", root_msg);
        
        // Analyze each process tree
        for (i, &root_pid) in root_pids.iter().enumerate() {
            if i > 0 {
                if self.no_color {
                    println!("\n{}", "=".repeat(60));
                } else {
                    println!();
                }
            }
            
            // Build and print tree
            if let Some(root_process) = self.build_process_tree(root_pid) {
                // Collect all RSS values in this tree and find max, second max, and third max
                let all_rss_in_tree = self.collect_all_rss_in_tree(&root_process);
                
                // Calculate total memory for this tree
                let total_memory = self.calculate_total_memory(&root_process);
                
                // Mark processes with max, second max, and third max memory
                if !all_rss_in_tree.is_empty() {
                    let tree_max_rss = *all_rss_in_tree.iter().max().unwrap();
                    let filtered_rss: Vec<u64> = all_rss_in_tree.iter().filter(|&&rss| rss != tree_max_rss).cloned().collect();
                    let tree_second_max_rss = if !filtered_rss.is_empty() {
                        *filtered_rss.iter().max().unwrap()
                    } else {
                        0
                    };
                    
                    // Find third max
                    let third_filtered_rss: Vec<u64> = filtered_rss.iter().filter(|&&rss| rss != tree_second_max_rss).cloned().collect();
                    let tree_third_max_rss = if !third_filtered_rss.is_empty() {
                        *third_filtered_rss.iter().max().unwrap()
                    } else {
                        0
                    };
                    
                    // Mark processes with max, second max, and third max memory
                    self.mark_memory_highlights_in_tree(root_pid, tree_max_rss, tree_second_max_rss, tree_third_max_rss);
                }
                
                // Get the updated root process after marking highlights
                if let Some(updated_root_process) = self.processes.get(&root_pid).cloned() {
                    self.print_tree(&updated_root_process, 0, false, total_memory);
                }
                
                // Print summary
                let process_count = self.count_processes(&root_process);
                
                // Calculate and print average memory
                let average_memory = if process_count > 0 {
                    total_memory / process_count as u64
                } else {
                    0
                };
                
                let avg_memory_str = self.get_colored_memory_str(average_memory, false, false, false);
                let total_memory_str = self.get_colored_memory_str(total_memory, false, false, false);
                let summary = if self.no_color {
                    format!("{} procs | {} avg | {} total", 
                            process_count, 
                            self.format_memory(average_memory), 
                            self.format_memory(total_memory))
                } else {
                    format!("{} {} procs | {} avg | {} total", 
                            colors::BOLD, process_count,
                            avg_memory_str, total_memory_str)
                };
                println!("{}", summary);
            } else {
                let error_msg = if self.no_color {
                    format!("Could not build process tree for PID {}", root_pid)
                } else {
                    format!("{}{}Could not build process tree for PID {}{}", 
                            colors::RED, colors::BOLD, root_pid, colors::RESET)
                };
                println!("{}", error_msg);
            }
        }
        
        Ok(true)
    }
    
    // Improved process name matching logic
    fn is_process_matching(&self, proc_name: &str, target_name: &str) -> bool {
        let proc_name_lower = proc_name.to_lowercase();
        let target_name_lower = target_name.to_lowercase();
        
        // Handle truncated process names (common on macOS with ps -c)
        // If target name is being searched and process name might be truncated
        if proc_name_lower.len() >= 15 && target_name_lower.starts_with(&proc_name_lower) {
            return true;
        }
        
        // Handle case where target name is long and might be truncated
        if target_name_lower.len() > 15 && proc_name_lower.starts_with(&target_name_lower[..15]) {
            return true;
        }
        
        // Exact match
        if proc_name_lower == target_name_lower {
            return true;
        }
        
        // Check if target name starts with process name (for truncated names)
        if target_name_lower.starts_with(&proc_name_lower) {
            return true;
        }
        
        // Check if process name starts with target name (for partial matching)
        if proc_name_lower.starts_with(&target_name_lower) {
            return true;
        }
        
        // Extract basename from process name if it contains a path
        let proc_basename = if proc_name_lower.contains('/') {
            proc_name_lower.split('/').last().unwrap_or(&proc_name_lower).to_string()
        } else {
            proc_name_lower.clone()
        };
        
        // Extract basename from target name if it contains a path
        let target_basename = if target_name_lower.contains('/') {
            target_name_lower.split('/').last().unwrap_or(&target_name_lower).to_string()
        } else {
            target_name_lower.clone()
        };
        
        // Handle common executable extensions
        let base_proc = if proc_basename.ends_with(".exe") || proc_basename.ends_with(".app") || 
                          proc_basename.ends_with(".bin") || proc_basename.ends_with(".run") {
            proc_basename[..proc_basename.len()-4].to_string()
        } else {
            proc_basename
        };
        
        let base_target = if target_basename.ends_with(".exe") || target_basename.ends_with(".app") || 
                            target_basename.ends_with(".bin") || target_basename.ends_with(".run") {
            target_basename[..target_basename.len()-4].to_string()
        } else {
            target_basename
        };
        
        if base_proc == base_target {
            return true;
        }
        
        // Check for common macOS app naming patterns
        // Some apps have process names like "App Name" when app is "AppName"
        if target_name.contains(' ') {
            let compact_name = target_name.replace(' ', "").to_lowercase();
            if proc_name_lower == compact_name {
                return true;
            }
        }
        
        false
    }
    
    // Count total number of processes in tree
    fn count_processes(&self, root: &ProcessInfo) -> usize {
        let mut count = 1; // Root itself
        for child_pid in &root.children {
            if let Some(child) = self.processes.get(child_pid) {
                count += self.count_processes(child);
            }
        }
        count
    }
    
    // Calculate total RSS memory for a process tree
    fn calculate_total_memory(&self, root: &ProcessInfo) -> u64 {
        let mut total_memory = root.rss; // Root's memory
        for child_pid in &root.children {
            if let Some(child) = self.processes.get(child_pid) {
                total_memory += self.calculate_total_memory(child);
            }
        }
        total_memory
    }
    
    // Collect all RSS values from processes in the tree
    fn collect_all_rss_in_tree(&self, root: &ProcessInfo) -> Vec<u64> {
        let mut rss_values = vec![root.rss]; // Root's RSS
        for child_pid in &root.children {
            if let Some(child) = self.processes.get(child_pid) {
                rss_values.extend(self.collect_all_rss_in_tree(child));
            }
        }
        rss_values
    }
    
    // Mark processes with max, second max, and third max memory in the tree
    fn mark_memory_highlights_in_tree(&mut self, root_pid: u32, max_rss: u64, second_max_rss: u64, third_max_rss: u64) {
        // Create a list of all process IDs in the tree to avoid borrowing issues
        let mut process_ids = Vec::new();
        self.collect_process_ids_in_tree(root_pid, &mut process_ids);
        
        // Mark processes with max, second max, and third max memory
        for pid in process_ids {
            if let Some(proc_info) = self.processes.get_mut(&pid) {
                if proc_info.rss == max_rss {
                    proc_info.is_max_memory = true;
                } else if proc_info.rss == second_max_rss && second_max_rss > 0 {
                    proc_info.is_second_max_memory = true;
                } else if proc_info.rss == third_max_rss && third_max_rss > 0 {
                    proc_info.is_third_max_memory = true;
                }
            }
        }
    }
    
    // Collect all process IDs in the tree
    fn collect_process_ids_in_tree(&self, root_pid: u32, process_ids: &mut Vec<u32>) {
        process_ids.push(root_pid);
        if let Some(proc_info) = self.processes.get(&root_pid) {
            for child_pid in &proc_info.children {
                self.collect_process_ids_in_tree(*child_pid, process_ids);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Create memory monitor and analyze
    let mut monitor = MemoryMonitor::new(!colors::should_use_colors(args.no_color));
    let success = monitor.analyze_process_tree(&args.process_name)?;
    
    if !success {
        std::process::exit(1);
    }
    
    Ok(())
}