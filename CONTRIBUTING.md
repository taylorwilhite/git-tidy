# Contributing to git-tidy

Thank you for your interest in contributing to git-tidy!

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/git-tidy.git
   cd git-tidy
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Running Tests

Run all tests:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

Run specific tests:
```bash
cargo test test_is_protected
```

## Code Style

Format code:
```bash
cargo fmt
```

Run linter:
```bash
cargo clippy
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and ensure they pass
5. Run `cargo fmt` and `cargo clippy`
6. Commit your changes
7. Push to the branch
8. Open a Pull Request

## Development Notes

- Use `anyhow` for error handling - no need for custom error types
- Keep the CLI simple and user-friendly
- Dry-run mode should be the default
- Always protect the current branch (HEAD)
- Never delete unmerged branches without explicit user intent
- Follow Rust naming conventions and best practices
