# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-08-30

### Added
- Initial release of memon-qwen
- Modern process tree memory analyzer with rich terminal output
- Cross-platform support (macOS, Linux)
- Color-coded memory usage levels
- Process name matching with various formats
- Watch mode for continuous monitoring
- Comprehensive test suite with pytest
- Development tools configuration (black, flake8, mypy)
- Virtual environment installation script
- Example usage scripts

### Changed
- Complete rewrite of original memon application
- Modernized codebase with better structure and practices
- Replaced custom ANSI color handling with rich library
- Simplified process tree visualization
- Improved error handling and user feedback

### Removed
- Legacy platform-specific code paths
- Custom argument parsing in favor of argparse
- Manual memory formatting in favor of rich library features