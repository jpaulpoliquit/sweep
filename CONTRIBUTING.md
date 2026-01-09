# Contributing to Wole

Thank you for your interest in contributing to Wole! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue using the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md). Include:

- A clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- System information (OS version, Rust version, etc.)
- Relevant logs or error messages

### Suggesting Features

Feature requests are welcome! Use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md) and include:

- A clear description of the feature
- Use case and motivation
- Potential implementation approach (if you have ideas)

### Pull Requests

1. **Fork the repository** and create a branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Follow the coding style (see below)
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

3. **Commit your changes**:
   ```bash
   git commit -m "feat: add your feature description"
   ```
   Use conventional commit messages (see below).

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Open a Pull Request**:
   - Use the PR template
   - Reference any related issues
   - Describe your changes clearly
   - Wait for review and address feedback

## Development Setup

### Prerequisites

- Rust (stable toolchain)
- Visual Studio Build Tools (Windows)
- Git

### Building

```powershell
# Debug build
cargo build

# Release build
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test --verbose

# Run specific test
cargo test --test integration_tests test_name
```

### Code Quality Checks

Before submitting a PR, ensure:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test
```

These checks run automatically in CI, but running them locally saves time.

## Coding Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Follow clippy suggestions (warnings are treated as errors in CI)
- Prefer explicit error handling over panics
- Document public APIs with doc comments

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

Examples:
```
feat: add support for custom cache directories
fix: handle locked files gracefully
docs: update installation instructions
```

### Project Structure

- `src/` - Main source code
- `src/categories/` - Category-specific cleanup logic
- `src/tui/` - Terminal UI components
- `tests/` - Integration tests
- `.github/` - GitHub workflows and templates

## Testing Guidelines

- Write tests for new features
- Test edge cases and error conditions
- Integration tests go in `tests/integration_tests.rs`
- Unit tests go in the same file as the code (using `#[cfg(test)]`)

## Documentation

- Update README.md for user-facing changes
- Add doc comments for public APIs
- Update CONTRIBUTING.md if workflow changes
- Keep code examples in docs up to date

## Review Process

1. All PRs require at least one review before merging
2. CI must pass (formatting, clippy, tests)
3. Address review comments promptly
4. Squash commits if requested during review

## Questions?

- Open an issue for discussion
- Check existing issues and PRs first
- Be patient and respectful

Thank you for contributing to Wole! 🎉
