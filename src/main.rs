// Memory Monitor - Process Tree Memory Analyzer
// Analyzes memory usage of a process and its children, displaying as a tree structure

use clap::Parser;
use std::collections::HashMap;
use sysinfo::System;

mod tests;

// ANSI color codes for cross-platform colored output
pub mod colors {
    // Reset
    pub const RESET: &str = "\x1b[0m";

    // Foreground colors
    pub const CYAN: &str = "\x1b[36m";

    // Background colors - light gray background
    pub const BG_LIGHT_GRAY: &str = "\x1b[47m"; // Light gray background

    // Foreground colors - dark gray for contrast
    pub const DARK_GRAY: &str = "\x1b[30m"; // Dark gray foreground

    pub fn should_use_colors(no_color_flag: bool) -> bool {
        if no_color_flag || std::env::var("NO_COLOR").is_ok() {
            return false;
        }
        true
    }
}

#[derive(Parser, Debug)]
#[clap(
    name = "memon",
    version = "0.1.0",
    author = "Your Name <you@example.com>",
    about = "Analyzes memory usage of a process and its children"
)]
pub struct Args {
    #[clap(name = "PROCESS_NAME")]
    process_name: String,

    #[clap(long)]
    verbose: bool,

    #[clap(short = 'v', long = "show-args")]
    show_args: bool,

    #[clap(long)]
    no_color: bool,

    #[clap(short, long)]
    watch: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub rss: u64,
    pub parent_pid: Option<u32>,
    pub children: Vec<u32>,
    pub is_max_memory: bool,
    pub is_second_max_memory: bool,
    pub is_third_max_memory: bool,
    pub args: Option<String>,
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

pub struct MemoryMonitor {
    processes: HashMap<u32, ProcessInfo>,
    no_color: bool,
    show_args: bool,
    system: System,
}

impl MemoryMonitor {
    pub fn new(no_color: bool, show_args: bool) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        MemoryMonitor {
            processes: HashMap::new(),
            no_color,
            show_args,
            system,
        }
    }

    pub fn get_all_processes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.system.refresh_processes();
        self.processes.clear();

        for (pid, process) in self.system.processes() {
            let pid_value = pid.as_u32();
            let name = process.name().to_string();
            let rss = process.memory();
            let ppid = process.parent().map(|p| p.as_u32());

            let args = if self.show_args {
                process.cmd().join(" ")
            } else {
                String::new()
            };

            let mut proc_info = ProcessInfo::new(pid_value, name, rss, ppid);
            if self.show_args && !args.is_empty() {
                let masked_args = MemoryMonitor::mask_sensitive_args(&args);
                proc_info.args = Some(masked_args);
            }

            self.processes.insert(pid_value, proc_info);
        }

        Ok(())
    }

    pub fn build_process_tree(&mut self, root_pid: u32) -> Option<ProcessInfo> {
        if !self.processes.contains_key(&root_pid) {
            return None;
        }

        for (_, proc_info) in self.processes.iter_mut() {
            proc_info.children.clear();
        }

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

    pub fn find_root_processes(&self, matching_pids: &[u32]) -> Vec<u32> {
        let mut root_pids = Vec::new();

        for &pid in matching_pids {
            if let Some(proc_info) = self.processes.get(&pid) {
                if let Some(parent_pid) = proc_info.parent_pid {
                    if !matching_pids.contains(&parent_pid)
                        || parent_pid == 1
                        || !self.processes.contains_key(&parent_pid)
                    {
                        root_pids.push(pid);
                    }
                } else {
                    root_pids.push(pid);
                }
            }
        }

        root_pids
    }

    pub fn format_memory(&self, bytes_value: u64) -> String {
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

    pub fn get_memory_color(
        &self,
        _bytes_value: u64,
        is_max_memory: bool,
        is_second_max_memory: bool,
        is_third_max_memory: bool,
    ) -> String {
        if self.no_color {
            return String::new();
        }

        if is_max_memory || is_second_max_memory || is_third_max_memory {
            return format!("{}{}", colors::DARK_GRAY, colors::BG_LIGHT_GRAY);
        }

        String::new()
    }

    pub fn get_colored_memory_str(
        &self,
        bytes_value: u64,
        is_max_memory: bool,
        is_second_max_memory: bool,
        is_third_max_memory: bool,
    ) -> String {
        let color = self.get_memory_color(
            bytes_value,
            is_max_memory,
            is_second_max_memory,
            is_third_max_memory,
        );
        let memory_str = self.format_memory(bytes_value);
        if self.no_color {
            memory_str
        } else {
            format!("{}{}{}", color, memory_str, colors::RESET)
        }
    }

    pub fn calculate_column_widths(&self, root: &ProcessInfo) -> (usize, usize) {
        let mut max_pid_width = 0;
        let mut max_name_width = 40;

        let mut all_processes = Vec::new();
        self.collect_all_processes_in_tree(root, &mut all_processes);

        for proc_info in &all_processes {
            let pid_str = proc_info.pid.to_string();
            max_pid_width = max_pid_width.max(pid_str.len());

            let display_name = if proc_info.name.len() > 40 {
                format!("{}...", &proc_info.name[..37])
            } else {
                proc_info.name.clone()
            };
            max_name_width = max_name_width.max(display_name.len());
        }

        (max_pid_width, max_name_width)
    }

    fn collect_all_processes_in_tree(&self, root: &ProcessInfo, processes: &mut Vec<ProcessInfo>) {
        processes.push(root.clone());
        for &child_pid in &root.children {
            if let Some(child) = self.processes.get(&child_pid) {
                self.collect_all_processes_in_tree(child, processes);
            }
        }
    }

    pub fn print_tree(
        &self,
        root: &ProcessInfo,
        level: usize,
        is_last: bool,
        total_memory: u64,
        pid_width: usize,
        name_width: usize,
    ) {
        let memory_str = self.get_colored_memory_str(
            root.rss,
            root.is_max_memory,
            root.is_second_max_memory,
            root.is_third_max_memory,
        );

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

        let rank_emoji = if root.is_max_memory {
            "🥇"
        } else if root.is_second_max_memory {
            "🥈"
        } else if root.is_third_max_memory {
            "🥉"
        } else {
            ""
        };

        let display_name = if root.name.len() > name_width {
            if name_width > 3 {
                format!("{}...", &root.name[..name_width - 3])
            } else {
                "...".to_string()
            }
        } else {
            format!("{:width$}", root.name, width = name_width)
        };

        print!("{}", tree_prefix);

        if self.show_args {
            print!("🟢");
        }

        print!(
            "{:width$} {} {}",
            root.pid,
            display_name,
            memory_str,
            width = pid_width
        );

        if let Some(ref args) = root.args {
            print!(" 🔍{}", args);
        }

        print!("{}", rank_emoji);

        println!();

        let child_count = root.children.len();
        for (i, child_pid) in root.children.iter().enumerate() {
            if let Some(child) = self.processes.get(child_pid) {
                self.print_tree(
                    child,
                    level + 1,
                    i == child_count - 1,
                    total_memory,
                    pid_width,
                    name_width,
                );
            }
        }
    }

    pub fn analyze_process_tree(
        &mut self,
        process_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let search_msg = if self.no_color {
            format!("Searching: {}", process_name)
        } else {
            format!(
                "Searching:{} {}{}",
                colors::CYAN,
                process_name,
                colors::RESET
            )
        };
        println!("{}", search_msg);

        self.get_all_processes()?;

        let matching_pids: Vec<u32> = self
            .processes
            .iter()
            .filter(|(_, proc_info)| self.is_process_matching(&proc_info.name, process_name))
            .map(|(&pid, _)| pid)
            .collect();

        if matching_pids.is_empty() {
            let not_found_msg = if self.no_color {
                format!("No processes found matching '{}'", process_name)
            } else {
                format!(
                    "No processes found matching '{}'{}",
                    process_name,
                    colors::RESET
                )
            };
            println!("{}", not_found_msg);
            return Ok(false);
        }

        let found_msg = if self.no_color {
            format!("Found {} procs", matching_pids.len())
        } else {
            format!("Found {} procs{}", matching_pids.len(), colors::RESET)
        };
        println!("{}", found_msg);

        let root_pids = self.find_root_processes(&matching_pids);

        if root_pids.is_empty() {
            println!("No root processes found");
            return Ok(false);
        }

        let root_msg = if self.no_color {
            format!("Found {} trees", root_pids.len())
        } else {
            format!("Found {} trees{}", root_pids.len(), colors::RESET)
        };
        println!("{}", root_msg);

        for (i, &root_pid) in root_pids.iter().enumerate() {
            if i > 0 {
                if self.no_color {
                    println!("\n{}", "=".repeat(60));
                } else {
                    println!();
                }
            }

            if let Some(root_process) = self.build_process_tree(root_pid) {
                let all_rss_in_tree = self.collect_all_rss_in_tree(&root_process);

                let total_memory = self.calculate_total_memory(&root_process);

                if !all_rss_in_tree.is_empty() {
                    let tree_max_rss = *all_rss_in_tree.iter().max().unwrap();
                    let filtered_rss: Vec<u64> = all_rss_in_tree
                        .iter()
                        .filter(|&&rss| rss != tree_max_rss)
                        .cloned()
                        .collect();
                    let tree_second_max_rss = if !filtered_rss.is_empty() {
                        *filtered_rss.iter().max().unwrap()
                    } else {
                        0
                    };

                    let third_filtered_rss: Vec<u64> = filtered_rss
                        .iter()
                        .filter(|&&rss| rss != tree_second_max_rss)
                        .cloned()
                        .collect();
                    let tree_third_max_rss = if !third_filtered_rss.is_empty() {
                        *third_filtered_rss.iter().max().unwrap()
                    } else {
                        0
                    };

                    self.mark_memory_highlights_in_tree(
                        root_pid,
                        tree_max_rss,
                        tree_second_max_rss,
                        tree_third_max_rss,
                    );
                }

                if let Some(updated_root_process) = self.processes.get(&root_pid).cloned() {
                    let (pid_width, name_width) =
                        self.calculate_column_widths(&updated_root_process);
                    self.print_tree(
                        &updated_root_process,
                        0,
                        false,
                        total_memory,
                        pid_width,
                        name_width,
                    );
                }

                let process_count = self.count_processes(&root_process);

                let average_memory = if process_count > 0 {
                    total_memory / process_count as u64
                } else {
                    0
                };

                let all_rss_in_tree = self.collect_all_rss_in_tree(&root_process);
                let tree_max_rss = *all_rss_in_tree.iter().max().unwrap_or(&0);
                let tree_second_max_rss = if all_rss_in_tree.len() > 1 {
                    *all_rss_in_tree
                        .iter()
                        .filter(|&&rss| rss != tree_max_rss)
                        .max()
                        .unwrap_or(&0)
                } else {
                    0
                };
                let tree_third_max_rss = if all_rss_in_tree.len() > 2 {
                    *all_rss_in_tree
                        .iter()
                        .filter(|&&rss| rss != tree_max_rss && rss != tree_second_max_rss)
                        .max()
                        .unwrap_or(&0)
                } else {
                    0
                };

                let has_top_memory =
                    tree_max_rss > 0 || tree_second_max_rss > 0 || tree_third_max_rss > 0;

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
                    format!(
                        "{} procs | {} avg | {} total",
                        process_count,
                        self.format_memory(average_memory),
                        self.format_memory(total_memory)
                    )
                } else {
                    format!(
                        "{} procs | {} avg | {} total",
                        process_count, avg_memory_str, total_memory_str
                    )
                };
                println!("{}", summary);
            } else {
                println!("Could not build process tree for PID {}", root_pid);
            }
        }

        Ok(true)
    }

    pub fn is_process_matching(&self, proc_name: &str, target_name: &str) -> bool {
        let proc_name_lower = proc_name.to_lowercase();
        let target_name_lower = target_name.to_lowercase();

        if proc_name_lower.len() >= 15 && target_name_lower.starts_with(&proc_name_lower) {
            return true;
        }

        if target_name_lower.len() > 15 && proc_name_lower.starts_with(&target_name_lower[..15]) {
            return true;
        }

        if proc_name_lower == target_name_lower {
            return true;
        }

        if target_name_lower.starts_with(&proc_name_lower) {
            return true;
        }

        if proc_name_lower.starts_with(&target_name_lower) {
            return true;
        }

        let proc_basename = if proc_name_lower.contains('/') {
            proc_name_lower
                .split('/')
                .last()
                .unwrap_or(&proc_name_lower)
                .to_string()
        } else {
            proc_name_lower.clone()
        };

        let target_basename = if target_name_lower.contains('/') {
            target_name_lower
                .split('/')
                .last()
                .unwrap_or(&target_name_lower)
                .to_string()
        } else {
            target_name_lower.clone()
        };

        let base_proc = if proc_basename.ends_with(".exe")
            || proc_basename.ends_with(".app")
            || proc_basename.ends_with(".bin")
            || proc_basename.ends_with(".run")
        {
            proc_basename[..proc_basename.len() - 4].to_string()
        } else {
            proc_basename
        };

        let base_target = if target_basename.ends_with(".exe")
            || target_basename.ends_with(".app")
            || target_basename.ends_with(".bin")
            || target_basename.ends_with(".run")
        {
            target_basename[..target_basename.len() - 4].to_string()
        } else {
            target_basename
        };

        if base_proc == base_target {
            return true;
        }

        if target_name.contains(' ') {
            let compact_name = target_name.replace(' ', "").to_lowercase();
            if proc_name_lower == compact_name {
                return true;
            }
        }

        false
    }

    pub fn count_processes(&self, root: &ProcessInfo) -> usize {
        let mut count = 1;
        for child_pid in &root.children {
            if let Some(child) = self.processes.get(child_pid) {
                count += self.count_processes(child);
            }
        }
        count
    }

    pub fn calculate_total_memory(&self, root: &ProcessInfo) -> u64 {
        let mut total_memory = root.rss;
        for child_pid in &root.children {
            if let Some(child) = self.processes.get(child_pid) {
                total_memory += self.calculate_total_memory(child);
            }
        }
        total_memory
    }

    fn collect_all_rss_in_tree(&self, root: &ProcessInfo) -> Vec<u64> {
        let mut rss_values = vec![root.rss];
        for child_pid in &root.children {
            if let Some(child) = self.processes.get(child_pid) {
                rss_values.extend(self.collect_all_rss_in_tree(child));
            }
        }
        rss_values
    }

    fn mark_memory_highlights_in_tree(
        &mut self,
        root_pid: u32,
        max_rss: u64,
        second_max_rss: u64,
        third_max_rss: u64,
    ) {
        let mut process_ids = Vec::new();
        self.collect_process_ids_in_tree(root_pid, &mut process_ids);

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

    fn collect_process_ids_in_tree(&self, root_pid: u32, process_ids: &mut Vec<u32>) {
        process_ids.push(root_pid);
        if let Some(proc_info) = self.processes.get(&root_pid) {
            for child_pid in &proc_info.children {
                self.collect_process_ids_in_tree(*child_pid, process_ids);
            }
        }
    }

    pub fn mask_sensitive_args(args: &str) -> String {
        let patterns = vec![
            ("--password ", ' ', 8),
            ("-p ", ' ', 1),
            (" --passwd ", ' ', 7),
            ("--token ", ' ', 6),
            ("--api-key ", ' ', 8),
            ("--apikey ", ' ', 7),
            ("--secret ", ' ', 8),
            (" --auth ", ' ', 5),
        ];

        let env_patterns = vec![
            ("PASSWORD=", 8),
            ("PASSWD=", 7),
            ("TOKEN=", 6),
            ("API_KEY=", 8),
            ("APIKEY=", 7),
            ("SECRET=", 7),
        ];

        let mut result = args.to_string();
        let lower_result = result.to_lowercase();

        for (pattern, delim, min_chars) in &patterns {
            if let Some(pos) = lower_result.find(pattern) {
                let value_start = pos + pattern.len();
                if value_start >= result.len() {
                    continue;
                }

                let remaining = &result[value_start..];
                if let Some(delim_pos) = remaining.find(*delim) {
                    if delim_pos > 0 {
                        let start = value_start;
                        let end = value_start + delim_pos;
                        let masked = "*".repeat(delim_pos);
                        result.replace_range(start..end, &masked);
                    }
                } else if !remaining.is_empty() {
                    let masked = "*".repeat(remaining.len().max(*min_chars));
                    result.replace_range(value_start.., &masked);
                }
            }
        }

        for (pattern, _) in &env_patterns {
            let lower_result = result.to_lowercase();
            if let Some(pos) = lower_result.find(&pattern.to_lowercase()) {
                let value_start = pos + pattern.len();
                if value_start >= result.len() {
                    continue;
                }

                let remaining = &result[value_start..];
                if let Some(space_pos) = remaining.find(' ') {
                    if space_pos > 0 {
                        let start = value_start;
                        let end = value_start + space_pos;
                        let masked = "*".repeat(space_pos);
                        result.replace_range(start..end, &masked);
                    }
                } else if !remaining.is_empty() {
                    let masked = "*".repeat(remaining.len());
                    result.replace_range(value_start.., &masked);
                }
            }
        }

        result
    }

    pub fn validate_process_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 255 {
            return false;
        }

        if name.chars().any(|c| c.is_control()) {
            return false;
        }

        true
    }

    pub fn run_with_watch<F>(
        &mut self,
        process_name: &str,
        interval_secs: u64,
        mut callback: F,
    ) -> Result<bool, Box<dyn std::error::Error>>
    where
        F: FnMut() + Clone,
    {
        loop {
            let result = self.analyze_process_tree(process_name)?;

            callback();

            if !result {
                return Ok(false);
            }

            std::thread::sleep(std::time::Duration::from_secs(interval_secs));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !MemoryMonitor::validate_process_name(&args.process_name) {
        eprintln!("Error: Invalid process name");
        std::process::exit(1);
    }

    let mut monitor = MemoryMonitor::new(!colors::should_use_colors(args.no_color), args.show_args);

    if let Some(interval) = args.watch {
        monitor.run_with_watch(&args.process_name, interval, || {})?;
    } else {
        let success = monitor.analyze_process_tree(&args.process_name)?;

        if !success {
            std::process::exit(1);
        }
    }

    Ok(())
}
