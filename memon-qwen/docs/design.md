# memon-qwen Design and Architecture

## Project Structure

```
memon-qwen/
├── src/
│   └── memon/
│       ├── __init__.py
│       └── main.py
├── tests/
│   └── test_memon.py
├── examples/
│   ├── example_usage.py
│   └── watch_demo.py
├── docs/
├── venv/
├── pyproject.toml
├── README.md
├── install.sh
├── test.sh
├── Makefile
├── mypy.ini
└── .flake8
```

## Key Design Decisions

### 1. Modern Python Packaging
- Uses `pyproject.toml` for project configuration
- Follows modern Python packaging standards (PEP 518, PEP 621)
- Supports both direct installation and development mode

### 2. Dependency Management
- Uses `psutil` for cross-platform process management
- Uses `rich` for modern terminal output with colors and tree structures
- Separates development dependencies from runtime dependencies

### 3. Code Structure
- Clean separation of concerns with `ProcessInfo` and `MemoryMonitor` classes
- Modular design with clear responsibilities for each component
- Type hints for better code documentation and IDE support

### 4. Testing
- Comprehensive unit tests with pytest
- Mock-based testing for external dependencies
- Coverage for key functionality

### 5. User Experience
- Modern CLI with intuitive commands
- Color-coded output for better readability
- Watch mode for continuous monitoring
- Helpful error messages and usage instructions

## Key Improvements Over Original Version

### 1. Better Process Tree Visualization
- Uses `rich` library for modern tree display
- Cleaner and more readable output format
- Color-coded memory usage indicators

### 2. Simplified Codebase
- Reduced complexity by leveraging `psutil`
- Eliminated platform-specific code paths
- Cleaner process matching logic

### 3. Enhanced Features
- Improved watch mode with live updates
- Better memory formatting and display
- More robust error handling

### 4. Modern Development Practices
- Proper package structure
- Comprehensive testing suite
- Type checking with mypy
- Code formatting with black
- Linting with flake8

## Usage Examples

### Basic Usage
```bash
# Activate virtual environment
source venv/bin/activate

# Analyze Python processes
memon python

# Analyze with no color output
memon chrome --no-color

# Watch mode
memon firefox -t 2

# Deactivate virtual environment
deactivate
```

### Development Workflow
```bash
# Run tests
make test

# Format code
make format

# Check types
make check-types

# Run all checks
make verify
```