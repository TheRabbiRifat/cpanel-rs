# Contributing to cpanel-rs

Thank you for your interest in contributing to `cpanel-rs`! We welcome contributions of all kinds, including bug reports, documentation improvements, feature requests, and code contributions.

---

## Code of Conduct

Please be respectful and constructive in all interactions within this project.

---

## Getting Started

1. **Fork** the repository on GitHub.
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/TheRabbiRifat/cpanel-rs.git
   cd cpanel-rs
   ```
3. **Create a branch** for your feature or bugfix:
   ```bash
   git checkout -b feat/my-new-feature
   ```

---

## Development Setup

Make sure you have a modern Rust toolchain installed (MSRV is Rust 1.70+):

```bash
# Update Rust toolchain
rustup update stable

# Build the project
cargo build

# Run all tests
cargo test --all-features

# Run Clippy linter
cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
cargo fmt --all -- --check

# Generate and verify documentation
cargo doc --no-deps --all-features
```

---

## Guidelines

- **Code Style:** Format your code using `cargo fmt` before submitting.
- **Documentation:** All public APIs must include doc comments with descriptions, examples, and `# Errors` sections where applicable.
- **Testing:** Add unit or integration tests for any new features or bug fixes. Ensure all existing tests pass.
- **Changelog:** Add an entry under the `[Unreleased]` section in `CHANGELOG.md` describing your changes.

---

## Pull Request Process

1. Ensure your branch is up-to-date with `main`.
2. Verify all checks pass locally:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo doc --no-deps --all-features
   ```
3. Open a Pull Request on GitHub against the `main` branch with a clear and concise description of the changes.

---

## Commit Messages

We encourage the use of [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add DNS zone import/export support
fix: resolve JSON deserialization error in backup list
docs: update README quickstart example
chore: update dependencies
```

---

## Reporting Issues

If you encounter a bug or have a feature request, please search existing issues or open a new one on the [GitHub Issue Tracker](https://github.com/TheRabbiRifat/cpanel-rs/issues) using the provided issue templates.

---

## License

By contributing to `cpanel-rs`, you agree that your contributions will be licensed under the [MIT License](LICENSE).

