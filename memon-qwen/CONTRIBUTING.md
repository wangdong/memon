# Contributing to memon-qwen

Thank you for your interest in contributing to memon-qwen! Here are some guidelines to help you get started.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/memon-qwen.git`
3. Create a virtual environment: `./install.sh`
4. Activate the virtual environment: `source venv/bin/activate`

## Development Workflow

### Running Tests

```bash
# Run all tests
pytest

# Run tests with coverage
pytest --cov=memon --cov-report=html

# Run specific test file
pytest tests/test_memon.py
```

### Code Formatting

```bash
# Format code with black
black .

# Check code formatting
black --check .
```

### Linting

```bash
# Check code style
flake8 .
```

### Type Checking

```bash
# Run mypy type checker
mypy src/
```

### Running All Checks

```bash
# Run all checks at once
make verify
```

## Making Changes

1. Create a new branch for your feature or bugfix: `git checkout -b feature/your-feature-name`
2. Make your changes
3. Add tests if applicable
4. Run all checks to ensure nothing is broken
5. Commit your changes: `git commit -am "Add your commit message"`
6. Push to your fork: `git push origin feature/your-feature-name`
7. Create a pull request

## Code Style

- Follow PEP 8 for Python code style
- Use type hints where possible
- Write docstrings for public functions and classes
- Keep functions small and focused
- Use descriptive variable names

## Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting a pull request
- Aim for high test coverage
- Use pytest for testing

## Documentation

- Update README.md if you change functionality
- Add docstrings to new functions and classes
- Update docs/design.md if you make significant architectural changes

## Reporting Issues

If you find a bug or have a feature request, please open an issue on GitHub with:

1. A clear title
2. A detailed description of the problem or feature
3. Steps to reproduce (for bugs)
4. Expected and actual behavior (for bugs)
5. Environment information (OS, Python version, etc.)

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add an entry to CHANGELOG.md if appropriate
4. Squash related commits
5. Write a clear commit message
6. Submit the pull request

Thank you for contributing!