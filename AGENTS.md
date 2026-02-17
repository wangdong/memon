# AGENTS.md

This file provides information for AI agents working on this codebase.

## Project Overview

- **Project name**: memon
- **Type**: CLI tool (Rust)
- **Purpose**: Memory monitor that analyzes process memory usage and displays results in a tree structure
- **Edition**: 2024

## Key Commands

```bash
# Build debug version
cargo build

# Build release version
cargo build --release

# Run application
cargo run -- <process_name>

# Run with specific flags
cargo run -- chrome -v --watch 5

# Check formatting
cargo fmt

# Lint code
cargo clippy
```

## Project Structure

```
memon/
├── src/main.rs          # All application logic (681 lines)
├── Cargo.toml           # Project configuration
├── README.md            # User documentation
└── AGENTS.md            # This file
```

## Dependencies

- `clap` 4.0 (with derive feature) - CLI argument parsing
- `sysinfo` 0.30 - System/process information

## Code Overview

### Entry Point
- `main()` in `src/main.rs:669`

### Key Structures
- `Args` - CLI arguments (lines 44-71)
- `ProcessInfo` - Process data struct (lines 74-85)
- `MemoryMonitor` - Main analyzer (lines 108-667)

### Features
- Process tree memory analysis with hierarchical display
- Memory ranking (top 3 processes with 🥇🥈🥉)
- Watch mode for continuous monitoring
- Colored output (configurable via `--no-color`)
- Process argument display (`-v` / `--show-args`)
- Smart process name matching (truncated names, extensions, etc.)
- Dynamic column alignment

### CLI Options
| Option | Description |
|--------|-------------|
| `PROCESS_NAME` | Process name to analyze (required) |
| `--verbose` | Enable verbose output |
| `-v, --show-args` | Display process startup arguments |
| `--no-color` | Disable colored output |
| `-w, --watch <SECONDS>` | Watch mode interval |

## Notes

- The project uses `edition = "2024"` in Cargo.toml (unstable/future edition)
- All code is contained in a single file (`src/main.rs`)
- No tests are defined in the current version