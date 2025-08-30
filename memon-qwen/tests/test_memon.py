"""
Tests for memon
"""

import pytest
from unittest.mock import Mock, patch
import psutil

from memon.main import ProcessInfo, MemoryMonitor


class TestProcessInfo:
    """Tests for ProcessInfo class"""
    
    def test_initialization(self):
        """Test ProcessInfo initialization"""
        proc = ProcessInfo(1234, "test_process", 1024, 2048, 1)
        
        assert proc.pid == 1234
        assert proc.name == "test_process"
        assert proc.rss == 1024
        assert proc.vsz == 2048
        assert proc.parent_pid == 1
        assert proc.children == []


class TestMemoryMonitor:
    """Tests for MemoryMonitor class"""
    
    def test_initialization(self):
        """Test MemoryMonitor initialization"""
        monitor = MemoryMonitor()
        
        assert monitor.no_color is False
        assert monitor.processes == {}
    
    def test_initialization_no_color(self):
        """Test MemoryMonitor initialization with no_color"""
        monitor = MemoryMonitor(no_color=True)
        
        assert monitor.no_color is True
        assert monitor.processes == {}
    
    def test_format_memory_bytes(self):
        """Test memory formatting in bytes"""
        monitor = MemoryMonitor()
        
        assert monitor._format_memory(0) == "0B"
        assert monitor._format_memory(1023) == "0.0MB"  # Less than 1KB
        
    def test_format_memory_mb(self):
        """Test memory formatting in MB"""
        monitor = MemoryMonitor()
        
        # 10MB
        assert monitor._format_memory(10 * 1024 * 1024) == "10.0MB"
        
        # 99.9MB
        assert monitor._format_memory(int(99.9 * 1024 * 1024)) == "99.9MB"
    
    def test_format_memory_gb(self):
        """Test memory formatting in GB"""
        monitor = MemoryMonitor()
        
        # 1GB
        assert monitor._format_memory(1024 * 1024 * 1024) == "1.0GB"
        
        # 2.5GB
        assert monitor._format_memory(int(2.5 * 1024 * 1024 * 1024)) == "2.5GB"
    
    def test_get_memory_color(self):
        """Test memory color coding"""
        monitor = MemoryMonitor()
        
        # Test with color enabled
        assert monitor._get_memory_color(5 * 1024 * 1024) == "green"      # 5MB
        assert monitor._get_memory_color(50 * 1024 * 1024) == "yellow"    # 50MB
        assert monitor._get_memory_color(250 * 1024 * 1024) == "magenta"  # 250MB
        assert monitor._get_memory_color(750 * 1024 * 1024) == "red"      # 750MB
    
    def test_get_memory_color_no_color(self):
        """Test memory color coding with no_color"""
        monitor = MemoryMonitor(no_color=True)
        
        # Test with color disabled
        assert monitor._get_memory_color(5 * 1024 * 1024) == ""      # 5MB
        assert monitor._get_memory_color(50 * 1024 * 1024) == ""     # 50MB
        assert monitor._get_memory_color(250 * 1024 * 1024) == ""    # 250MB
        assert monitor._get_memory_color(750 * 1024 * 1024) == ""    # 750MB
    
    @patch('psutil.process_iter')
    def test_find_processes_by_name(self, mock_process_iter):
        """Test finding processes by name"""
        # Create mock processes
        mock_processes = [
            Mock(info={'pid': 1234, 'name': 'firefox'}),
            Mock(info={'pid': 5678, 'name': 'chrome'}),
            Mock(info={'pid': 9012, 'name': 'firefox-developer'}),
            Mock(info={'pid': 3456, 'name': 'safari'}),
        ]
        
        mock_process_iter.return_value = mock_processes
        monitor = MemoryMonitor()
        
        # Find processes matching 'firefox'
        matching_pids = monitor._find_processes_by_name('firefox')
        
        # Should find 2 processes (firefox and firefox-developer)
        assert len(matching_pids) == 2
        assert 1234 in matching_pids
        assert 9012 in matching_pids
    
    @patch('psutil.Process')
    def test_find_root_processes(self, mock_process):
        """Test finding root processes"""
        # Create mock process objects
        mock_proc1 = Mock()
        mock_proc1.ppid.return_value = 1  # Root process
        
        mock_proc2 = Mock()
        mock_proc2.ppid.return_value = 1234  # Child process
        
        mock_proc3 = Mock()
        mock_proc3.ppid.return_value = 1  # Another root process
        
        # Mock the Process constructor to return different objects based on PID
        mock_process.side_effect = lambda pid: {
            1234: mock_proc1,
            5678: mock_proc2,
            9012: mock_proc3
        }[pid]
        
        monitor = MemoryMonitor()
        
        # Test with matching PIDs where some are root processes
        matching_pids = [1234, 5678, 9012]
        root_pids = monitor._find_root_processes(matching_pids)
        
        # Should find 2 root processes (1234 and 9012)
        assert len(root_pids) == 2
        assert 1234 in root_pids
        assert 9012 in root_pids