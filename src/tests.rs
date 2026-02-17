#[cfg(test)]
mod tests {
    use crate::{colors, MemoryMonitor, ProcessInfo};

    #[test]
    fn test_mask_sensitive_args_no_password() {
        let args = "/usr/bin/chrome --version";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert_eq!(result, args);
    }

    #[test]
    fn test_mask_sensitive_args_with_password() {
        let args = "/usr/bin/chrome --password secret123";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("--password ********"));
    }

    #[test]
    fn test_mask_sensitive_args_with_short_p() {
        let args = "/usr/bin/app -p mysecret";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("-p *******"));
    }

    #[test]
    fn test_mask_sensitive_args_with_token() {
        let args = "myapp --token abc123xyz";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("--token *********"));
    }

    #[test]
    fn test_mask_sensitive_args_with_api_key() {
        let args = "curl https://api --apikey sk-1234567890";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("--apikey ************"));
    }

    #[test]
    fn test_mask_sensitive_args_with_secret() {
        let args = "java -jar app.jar --secret mySecretKey";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("--secret ") && result != args);
    }

    #[test]
    fn test_mask_sensitive_args_with_env_style() {
        let args = "python app.py PASSWORD=secret123";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("PASSWORD=") && result != args);
    }

    #[test]
    fn test_mask_sensitive_args_multiple_patterns() {
        let args = "curl -X POST --password pass123 --token abc https://example.com";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("--password ") && result != args);
        assert!(!result.contains("pass123"));
    }

    #[test]
    fn test_mask_sensitive_args_case_insensitive() {
        let args = "/app --PASSWORD Secret123";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("--PASSWORD ") && result != args);
    }

    #[test]
    fn test_mask_sensitive_args_with_token_env() {
        let args = "go run main.go TOKEN=myToken123";
        let result = MemoryMonitor::mask_sensitive_args(args);
        assert!(result.contains("TOKEN=") && result != args);
    }

    #[test]
    fn test_validate_process_name_valid() {
        assert!(MemoryMonitor::validate_process_name("chrome"));
        assert!(MemoryMonitor::validate_process_name("google-chrome"));
        assert!(MemoryMonitor::validate_process_name("ChromeBrowser"));
    }

    #[test]
    fn test_validate_process_name_empty() {
        assert!(!MemoryMonitor::validate_process_name(""));
    }

    #[test]
    fn test_validate_process_name_too_long() {
        let long_name = "a".repeat(256);
        assert!(!MemoryMonitor::validate_process_name(&long_name));
    }

    #[test]
    fn test_validate_process_name_with_control_char() {
        assert!(!MemoryMonitor::validate_process_name("chrome\x00"));
        assert!(!MemoryMonitor::validate_process_name("chrome\x01"));
    }

    #[test]
    fn test_validate_process_name_boundary() {
        let exactly_255 = "a".repeat(255);
        assert!(MemoryMonitor::validate_process_name(&exactly_255));
    }

    #[test]
    fn test_is_process_matching_exact() {
        let monitor = MemoryMonitor::new(true, false);
        assert!(monitor.is_process_matching("chrome", "chrome"));
        assert!(monitor.is_process_matching("Chrome", "chrome"));
        assert!(monitor.is_process_matching("CHROME", "chrome"));
    }

    #[test]
    fn test_is_process_matching_partial() {
        let monitor = MemoryMonitor::new(true, false);
        assert!(monitor.is_process_matching("chrome-bin", "chrome"));
    }

    #[test]
    fn test_format_memory_bytes() {
        let monitor = MemoryMonitor::new(true, false);
        assert_eq!(monitor.format_memory(0), "0B");
        assert_eq!(monitor.format_memory(1024 * 1024), "1.0MB");
        assert_eq!(monitor.format_memory(512 * 1024), "0.5MB");
    }

    #[test]
    fn test_format_memory_gigabytes() {
        let monitor = MemoryMonitor::new(true, false);
        assert_eq!(monitor.format_memory(1024 * 1024 * 1024), "1.0GB");
        assert_eq!(monitor.format_memory(2 * 1024 * 1024 * 1024), "2.0GB");
    }

    #[test]
    fn test_process_info_new() {
        let proc = ProcessInfo::new(1234, "test".to_string(), 1024000, Some(1));
        assert_eq!(proc.pid, 1234);
        assert_eq!(proc.name, "test");
        assert_eq!(proc.rss, 1024000);
        assert_eq!(proc.parent_pid, Some(1));
        assert!(proc.children.is_empty());
        assert!(!proc.is_max_memory);
    }

    #[test]
    fn test_process_info_add_child() {
        let mut proc = ProcessInfo::new(1, "parent".to_string(), 1000, None);
        proc.add_child(2);
        proc.add_child(3);
        assert_eq!(proc.children.len(), 2);
        assert!(proc.children.contains(&2));
        assert!(proc.children.contains(&3));
    }

    #[test]
    fn test_count_processes_single() {
        let monitor = MemoryMonitor::new(true, false);

        let root = ProcessInfo::new(1, "root".to_string(), 1000, None);

        assert_eq!(monitor.count_processes(&root), 1);
    }

    #[test]
    fn test_count_processes_tree() {
        let mut monitor = MemoryMonitor::new(true, false);

        let root = ProcessInfo::new(1, "root".to_string(), 1000, None);
        let child = ProcessInfo::new(2, "child1".to_string(), 500, Some(1));

        monitor.processes.insert(1, root);
        monitor.processes.insert(2, child);
        monitor.processes.get_mut(&1).unwrap().children.push(2);

        let root = monitor.processes.get(&1).unwrap();
        let count = monitor.count_processes(root);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_total_memory() {
        let mut monitor = MemoryMonitor::new(true, false);

        let root = ProcessInfo::new(1, "root".to_string(), 1000, None);
        let child = ProcessInfo::new(2, "child".to_string(), 500, Some(1));

        monitor.processes.insert(1, root);
        monitor.processes.insert(2, child);
        monitor.processes.get_mut(&1).unwrap().children.push(2);

        let root = monitor.processes.get(&1).unwrap();
        let total = monitor.calculate_total_memory(root);
        assert_eq!(total, 1500);
    }

    #[test]
    fn test_collect_all_rss_in_tree() {
        let mut monitor = MemoryMonitor::new(true, false);

        let root = ProcessInfo::new(1, "root".to_string(), 1000, None);
        let child = ProcessInfo::new(2, "child".to_string(), 500, Some(1));

        monitor.processes.insert(1, root);
        monitor.processes.insert(2, child);
        monitor.processes.get_mut(&1).unwrap().children.push(2);

        let root = monitor.processes.get(&1).unwrap();
        let rss_values = monitor.collect_all_rss_in_tree(root);
        assert_eq!(rss_values.len(), 2);
        assert!(rss_values.contains(&1000));
        assert!(rss_values.contains(&500));
    }

    #[test]
    fn test_colors_should_use_colors_no_color_flag() {
        assert!(!colors::should_use_colors(true));
    }

    #[test]
    fn test_colors_should_use_colors_no_color_env() {
        unsafe {
            std::env::set_var("NO_COLOR", "1");
            let result = colors::should_use_colors(false);
            std::env::remove_var("NO_COLOR");
            assert!(!result);
        }
    }

    #[test]
    fn test_get_colored_memory_str_no_color_mode() {
        let monitor = MemoryMonitor::new(true, false);
        let result = monitor.get_colored_memory_str(1024 * 1024, false, false, false);
        assert!(!result.contains("\x1b"));
    }

    #[test]
    fn test_get_colored_memory_str_with_rank() {
        let monitor = MemoryMonitor::new(false, false);

        let max_result = monitor.get_colored_memory_str(1024 * 1024, true, false, false);
        assert!(max_result.contains("\x1b"));

        let second_result = monitor.get_colored_memory_str(512 * 1024, false, true, false);
        assert!(second_result.contains("\x1b"));
    }

    #[test]
    fn test_sensitive_info_masked_in_show_args() {
        let mut monitor = MemoryMonitor::new(false, true);

        if let Err(_) = monitor.get_all_processes() {
            return;
        }

        for (_, proc) in monitor.processes.iter() {
            if let Some(ref args) = proc.args {
                assert!(!args.contains("secret"));
            }
        }
    }

    #[test]
    fn test_watch_mode_function_exists() {
        let mut monitor = MemoryMonitor::new(true, false);

        match monitor.run_with_watch("nonexistent", 1, || {}) {
            Ok(_) => {}
            Err(e) => {
                let err_msg = e.to_string();
                assert!(err_msg.contains("No processes found") || err_msg.is_empty());
            }
        }
    }
}
