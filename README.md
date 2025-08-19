# memon

A process tree memory analyzer that displays memory usage of a process and its children in a tree structure.

## Features

- Analyzes memory usage of processes and their children
- Displays results in a colored tree structure
- Shows both individual process memory (RSS) and total subtree memory (Total)
- Cross-platform support (macOS, Linux)
- Color-coded memory usage levels
- Process name matching with various formats

## Usage

```bash
python3 memon.py <process_name> [options]
```

### Options

- `-v, --verbose`: Enable verbose output
- `--no-color`: Disable colored output

### Examples

```bash
# Analyze Safari process tree
python3 memon.py Safari

# Analyze Chrome without colors
python3 memon.py Chrome --no-color

# Verbose output for Firefox
python3 memon.py Firefox -v
```

## Output Format

The output shows:
- Process ID and name
- RSS (Resident Set Size) - individual process memory
- Total - cumulative memory including all children
- Color-coded memory levels (green < 10MB, yellow < 100MB, magenta < 500MB, red >= 500MB)

## Requirements

- Python 3.6+
- macOS or Linux system

## License

This project is open source and available under the MIT License.