# Memon

A memory monitor tool that analyzes memory usage of processes and their children, displaying results as a tree structure.

## Features

- **Process Tree Analysis**: Displays process memory usage in a hierarchical tree format
- **Cross-Platform**: Built with Rust and the `sysinfo` crate for compatibility across different operating systems
- **Memory Ranking**: Highlights the top 3 memory-consuming processes with visual indicators
- **Watch Mode**: Continuously monitor memory usage with automatic updates
- **Colored Output**: Enhanced readability with color-coded memory usage (configurable)
- **Smart Process Matching**: Flexible process name matching that handles truncated names and common executable extensions

## Installation

### Prerequisites

- Rust (edition 2021 or later)
- Cargo (Rust's package manager)

### Build from Source

```bash
git clone https://github.com/yourusername/memon.git
cd memon
cargo build --release
```

The compiled binary will be available in `target/release/memon`.

## Usage

### Basic Usage

```bash
memon <PROCESS_NAME>
```

### Examples

```bash
# Analyze memory usage of a process named "chrome"
memon chrome

# Analyze with verbose output
memon chrome --verbose

# Disable colored output
memon chrome --no-color

# Watch mode - update every 5 seconds
memon chrome --watch 5

# Show help
memon --help

# Show version
memon --version
```

### Command Line Options

- `PROCESS_NAME`: Name of the process to analyze (required)
- `-v, --verbose`: Enable verbose output
- `--no-color`: Disable colored output
- `-w, --watch <SECONDS>`: Watch mode - continuously update every N seconds
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Output

Memon displays process information in a tree structure with the following format:

```
PID  PROCESS_NAME                              MEMORY   RANK
├─ 1234 chrome                                   2.5GB   🥇
├─ 1235 chrome                                   1.8GB   🥈
└─ 1236 chrome                                   1.2GB   🥉
```

### Output Elements

- **PID**: Process ID
- **PROCESS_NAME**: Process name (truncated to 40 characters)
- **MEMORY**: Memory usage in human-readable format (MB/GB)
- **RANK**: Visual indicator for top 3 memory consumers:
  - 🥇 Highest memory usage
  - 🥈 Second highest memory usage
  - 🥉 Third highest memory usage

### Memory Highlighting

The top 3 memory-consuming processes are highlighted with a light gray background and dark gray text for better visibility.

## Process Matching

Memon uses intelligent process name matching that supports:

- Exact matches
- Partial matches (process name starts with search term)
- Truncated process names (common on macOS)
- Executable extensions (.exe, .app, .bin, .run)
- Path basename matching
- macOS app naming patterns

## Dependencies

- `clap`: Command line argument parsing
- `sysinfo`: System information and process monitoring

## Development

### Project Structure

```
memon/
├── src/
│   └── main.rs          # Main application logic
├── Cargo.toml           # Project configuration
└── README.md            # This file
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt

# Lint code
cargo clippy
```

### Running in Development

```bash
cargo run -- chrome
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Changelog

### Version 0.1.0

- Initial release
- Process tree memory analysis
- Cross-platform compatibility using sysinfo
- Colored output with memory ranking
- Watch mode for continuous monitoring
- Smart process name matching