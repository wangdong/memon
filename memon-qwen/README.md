# memon-qwen

A modern process tree memory analyzer that displays memory usage of a process and its children in a tree structure.

## Features

- Analyzes memory usage of processes and their children
- Displays results in a modern, colored tree structure using [Rich](https://github.com/Textualize/rich)
- Shows both individual process memory (RSS) and total subtree memory
- Cross-platform support (macOS, Linux)
- Color-coded memory usage levels
- Process name matching with various formats
- Watch mode for continuous monitoring
- Modern CLI with intuitive commands

## Installation

### From PyPI

```bash
pip install memon-qwen
```

### From source

```bash
git clone https://github.com/yourusername/memon-qwen.git
cd memon-qwen
./install.sh
```

This will create a virtual environment and install all dependencies.

To use the tool after installation:

```bash
# Activate the virtual environment
source venv/bin/activate

# Run memon
memon <process_name>

# Deactivate when done
deactivate
```

## Usage

```bash
memon <process_name> [options]
```

### Options

- `-v, --verbose`: Enable verbose output
- `--no-color`: Disable colored output
- `-t, --watch SECONDS`: Watch mode - continuously update every N seconds

### Examples

```bash
# Analyze Safari process tree
memon Safari

# Analyze Chrome without colors
memon Chrome --no-color

# Watch Firefox memory usage every 2 seconds
memon Firefox -t 2

# Verbose output for Terminal
memon Terminal -v
```

## Development

### Setup

```bash
git clone https://github.com/yourusername/memon-qwen.git
cd memon-qwen
./install.sh
source venv/bin/activate
```

### Running tests

```bash
pytest
```

### Code formatting

```bash
black .
```

### Type checking

```bash
mypy src/
```

## Requirements

- Python 3.8+
- macOS or Linux system

## License

This project is open source and available under the MIT License.