# Makefile for memon-qwen

.PHONY: install test clean format check-types

# Install the package in development mode
install:
	pip install -e ".[dev]"

# Run tests
test:
	pytest

# Run tests with coverage
test-cov:
	pytest --cov=memon --cov-report=html --cov-report=term

# Format code with black
format:
	black .

# Check code formatting with black (dry run)
format-check:
	black --check .

# Check code style with flake8
lint:
	flake8 .

# Run type checking with mypy
check-types:
	mypy src/

# Clean generated files
clean:
	rm -rf *.egg-info
	rm -rf build
	rm -rf dist
	rm -rf .pytest_cache
	rm -rf .mypy_cache
	rm -rf htmlcov
	rm -rf .coverage
	find . -type d -name __pycache__ -delete
	find . -type f -name "*.pyc" -delete

# Run all checks
check: format-check lint check-types

# Run all tests and checks
verify: test check