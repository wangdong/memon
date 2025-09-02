// Memory Monitor - Process Tree Memory Analyzer
// Analyzes memory usage of a process and its children, displaying as a tree structure

use clap::Parser;
use std::collections::HashMap;
use sysinfo::System;

// ANSI color codes for cross-platform colored output
mod colors {
    // Reset
    pub const RESET: &str = "\x1b[0m";
    
    // Foreground colors
    pub const CYAN: &str = "\x1b[36m";
    
    // Background colors - light gray background
    pub const BG_LIGHT_GRAY: &str = "\x1b[47m";  // Light gray background
    
    // Foreground colors - dark gray for contrast
    pub const DARK_GRAY: &str = "\x1b[30m";  // Dark gray foreground
    
    // Styles - removed bold for cleaner output
    // pub const BOLD: &str = "\1b[1m"; // Removed
    
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
    
    // Functions to combine colors - removed as no longer used
    // pub fn combine_colors(color1: &str, color2: &str) -> String {
    //     format!("{}{}", color1, color2)
    // }
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
    #[clap(long)]
    verbose: bool,
    
    /// Display process startup arguments
    #[clap(short = 'v', long = "show-args")]
    show_args: bool,
    
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
    args: Option<String>, // Command line arguments
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
            args: None,
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
    show_args: bool,
    system: System,
}

impl MemoryMonitor {
    fn new(no_color: bool, show_args: bool) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        MemoryMonitor {
            processes: HashMap::new(),
            no_color,
            show_args,
            system,
        }
    }
    
    // Get all processes using sysinfo crate
    fn get_all_processes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Refresh system information
        self.system.refresh_processes();
        
        // Clear existing processes to avoid duplicates
        self.processes.clear();
        
        // Iterate through all processes
        for (pid, process) in self.system.processes() {
            let pid_value = pid.as_u32();
            let name = process.name().to_string();
            let rss = process.memory(); // Already in bytes
            let ppid = process.parent().map(|p| p.as_u32());
            
            // Get command line arguments if show_args is enabled
            let args = if self.show_args {
                process.cmd().join(" ")
            } else {
                String::new()
            };
            
            let mut proc_info = ProcessInfo::new(pid_value, name, rss, ppid);
            if self.show_args && !args.is_empty() {
                proc_info.args = Some(args);
            }
            
            self.processes.insert(pid_value, proc_info);
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
        
        // Use dark gray text with light gray background for top 1-3 memory processes
        if is_max_memory || is_second_max_memory || is_third_max_memory {
            // Dark gray text on light gray background
            return format!("{}{}", colors::DARK_GRAY, colors::BG_LIGHT_GRAY);
        }
        
        // No special color for non-trophy processes
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
    
    // Calculate column widths for proper alignment
    fn calculate_column_widths(&self, root: &ProcessInfo) -> (usize, usize) {
        let mut max_pid_width = 0;
        let mut max_name_width = 40; // Default minimum width
        
        // Collect all processes in the tree
        let mut all_processes = Vec::new();
        self.collect_all_processes_in_tree(root, &mut all_processes);
        
        // Find maximum PID width and process name width
        for proc_info in &all_processes {
            let pid_str = proc_info.pid.to_string();
            max_pid_width = max_pid_width.max(pid_str.len());
            
            // Calculate actual display name width
            let display_name = if proc_info.name.len() > 40 {
                format!("{}...", &proc_info.name[..37])
            } else {
                proc_info.name.clone()
            };
            max_name_width = max_name_width.max(display_name.len());
        }
        
        (max_pid_width, max_name_width)
    }
    
    // Collect all processes in the tree for width calculation
    fn collect_all_processes_in_tree(&self, root: &ProcessInfo, processes: &mut Vec<ProcessInfo>) {
        processes.push(root.clone());
        for &child_pid in &root.children {
            if let Some(child) = self.processes.get(&child_pid) {
                self.collect_all_processes_in_tree(child, processes);
            }
        }
    }
    
    // Print process tree with memory information
    fn print_tree(&self, root: &ProcessInfo, level: usize, is_last: bool, total_memory: u64, pid_width: usize, name_width: usize) {
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
            let mut prefix = String::new();
            for _ in 0..(level - 1) {
                prefix.push_str("  ");
            }
            prefix.push_str(if is_last { "└─ " } else { "├─ " });
            prefix
        } else {
            String::new()
        };
        
        // No PID coloring for cleaner output
        let _pid_color = "";
        
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
        
        // Truncate or pad process name to dynamic width
        let display_name = if root.name.len() > name_width {
            if name_width > 3 {
                format!("{}...", &root.name[..name_width-3])
            } else {
                "...".to_string()
            }
        } else {
            format!("{:width$}", root.name, width = name_width)
        };

        // Print process info with dynamic column widths
        print!("{}", tree_prefix);
        
        // Display green dot emoji before PID if show_args is enabled
        if self.show_args {
            print!("🟢");
        }
        
        print!("{:width$} {} {}", root.pid, display_name, memory_str, width = pid_width);
        
        // Display arguments if available
        if let Some(ref args) = root.args {
            print!(" 🔍{}", args);
        }
        
        // Display the rank emoji
        print!("{}", rank_emoji);
        
        // Print new line
        println!();
        
        // Print children
        let child_count = root.children.len();
        for (i, child_pid) in root.children.iter().enumerate() {
            if let Some(child) = self.processes.get(child_pid) {
                self.print_tree(child, level + 1, i == child_count - 1, total_memory, pid_width, name_width);
            }
        }
    }
    
    // Main analysis function
    fn analyze_process_tree(&mut self, process_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let search_msg = if self.no_color {
            format!("Searching: {}", process_name)
        } else {
            format!("Searching:{} {}{}", 
                    colors::CYAN, process_name, colors::RESET)
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
                format!("No processes found matching '{}'{}", 
                        process_name, colors::RESET)
            };
            println!("{}", not_found_msg);
            return Ok(false);
        }
        
        let found_msg = if self.no_color {
            format!("Found {} procs", matching_pids.len())
        } else {
            format!("Found {} procs{}", 
                    matching_pids.len(), colors::RESET)
        };
        println!("{}", found_msg);
        
        // Find root processes
        let root_pids = self.find_root_processes(&matching_pids);
        
        if root_pids.is_empty() {
            let no_root_msg = if self.no_color {
                "No root processes found".to_string()
            } else {
                "No root processes found".to_string()
            };
            println!("{}", no_root_msg);
            return Ok(false);
        }
        
        let root_msg = if self.no_color {
            format!("Found {} trees", root_pids.len())
        } else {
            format!("Found {} trees{}", 
                    root_pids.len(), colors::RESET)
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
                    // Calculate column widths for proper alignment
                    let (pid_width, name_width) = self.calculate_column_widths(&updated_root_process);
                    self.print_tree(&updated_root_process, 0, false, total_memory, pid_width, name_width);
                }
                
                // Print summary
                let process_count = self.count_processes(&root_process);
                
                // Calculate and print average memory
                let average_memory = if process_count > 0 {
                    total_memory / process_count as u64
                } else {
                    0
                };
                
                // For summary, we need to check if this tree contains top 3 memory processes
                let all_rss_in_tree = self.collect_all_rss_in_tree(&root_process);
                let tree_max_rss = *all_rss_in_tree.iter().max().unwrap_or(&0);
                let tree_second_max_rss = if all_rss_in_tree.len() > 1 {
                    *all_rss_in_tree.iter().filter(|&&rss| rss != tree_max_rss).max().unwrap_or(&0)
                } else { 0 };
                let tree_third_max_rss = if all_rss_in_tree.len() > 2 {
                    *all_rss_in_tree.iter().filter(|&&rss| rss != tree_max_rss && rss != tree_second_max_rss).max().unwrap_or(&0)
                } else { 0 };
                
                let has_top_memory = tree_max_rss > 0 || tree_second_max_rss > 0 || tree_third_max_rss > 0;
                
                let avg_memory_str = if has_top_memory {
                    self.get_colored_memory_str(average_memory, true, true, true)
                } else {
                    self.format_memory(average_memory)
                };
                let total_memory_str = if has_top_memory {
                    self.get_colored_memory_str(total_memory, true, true, true)
                } else {
                    self.format_memory(total_memory)
                };
                
                let summary = if self.no_color {
                    format!("{} procs | {} avg | {} total", 
                            process_count, 
                            self.format_memory(average_memory), 
                            self.format_memory(total_memory))
                } else {
                    format!("{} procs | {} avg | {} total", 
                            process_count,
                            avg_memory_str, total_memory_str)
                };
                println!("{}", summary);
            } else {
                let error_msg = if self.no_color {
                    format!("Could not build process tree for PID {}", root_pid)
                } else {
                    format!("Could not build process tree for PID {}", root_pid)
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
    let mut monitor = MemoryMonitor::new(!colors::should_use_colors(args.no_color), args.show_args);
    let success = monitor.analyze_process_tree(&args.process_name)?;
    
    if !success {
        std::process::exit(1);
    }
    
    Ok(())
}