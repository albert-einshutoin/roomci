# Contributing to roomci

Thank you for your interest in contributing to roomci! This document provides guidelines and instructions for contributing.

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Please read and adhere to our Code of Conduct:

- Be respectful and constructive in all interactions
- Welcome people of all backgrounds and perspectives
- Focus on criticism of ideas, not individuals
- Report unacceptable behavior to maintainers

## How to Contribute

### Reporting Issues

Before creating a bug report, check the issue list as you might find out that you don't need to create one. When creating a bug report, include as many details as possible:

- **Use a clear, descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples to demonstrate the steps**
- **Describe the observed behavior and what you expected instead**
- **Include screenshots or error messages if applicable**
- **Mention your environment** (OS, Rust version, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub Issues. When creating an enhancement suggestion:

- **Use a clear, descriptive title**
- **Provide a detailed description of the suggested enhancement**
- **Explain why this enhancement would be useful**
- **List some examples of where the enhancement would be used**

### Branching Model

roomci uses GitHub Flow:

- `main` is the only long-lived integration and release branch
- Create short-lived feature or fix branches from the latest `main`
- Open pull requests back into `main`
- Do not create or target a long-lived `develop` branch

### Pull Requests

1. **Fork the repository** and create your feature branch
   ```bash
   git clone https://github.com/yourusername/roomci.git
   cd roomci
   git checkout -b feature/your-feature-name
   ```

2. **Set up your development environment**
   ```bash
   cargo build
   cargo test
   make verify
   ```

3. **Make your changes**
   - Keep commits focused and atomic
   - Write clear commit messages (see Commit Message Format below)
   - Add tests for new functionality
   - Update documentation as needed

4. **Verify your changes**
   ```bash
   cargo test              # Run all tests
   cargo clippy           # Check for common mistakes
   cargo fmt --check      # Check formatting
   make verify            # Run full verification suite
   ```

5. **Submit your Pull Request**
   - Push your feature branch to your fork
   - Open a PR against `main` branch
   - Fill in the PR template with:
     - Description of changes
     - Link to relevant issues
     - Test plan
     - Screenshots (if applicable)

## Commit Message Format

Follow conventional commits format for clarity:

```
<type>: <subject>

<optional body>

<optional footer>
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **test**: Adding or updating tests
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Code change that improves performance
- **ci**: Changes to CI/CD configuration
- **chore**: Build process, dependency updates, etc.

### Examples

```
feat: add support for Zigbee protocol profiles

- Implement ZigbeeContractProfile
- Add example zigbee_gateway_profile.yaml
- Include integration tests

Fixes #123
```

```
fix: handle concurrent BMS alert subscriptions

Previously, concurrent alerts would overwrite pending state.
Now uses atomic CAS operation for safe concurrent updates.

Closes #456
```

## Development Setup

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Docker (for running compose tests)
- Make

### Quick Start

```bash
# Clone and setup
git clone https://github.com/yourusername/roomci.git
cd roomci

# Build
cargo build

# Run tests
cargo test

# Run full verification
make verify

# Try a scenario
cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml --verbose
```

### Project Structure

```
roomci/
├── crates/
│   ├── roomci-cli/           # Command-line interface
│   ├── roomci-scenario/      # Scenario execution engine
│   └── roomci-sdk/           # Reference SDK
├── adapter-contracts/        # Protocol profile definitions
├── examples/                 # Scenario examples and adapters
├── docs/                     # Documentation
├── tests/                    # Integration tests
└── compose/                  # Docker Compose setup
```

## Testing Guidelines

- **Unit tests**: Test individual functions and components
- **Integration tests**: Test end-to-end scenarios
- **Contract tests**: Validate adapter contracts

Target coverage: 80%+

```bash
# Run with coverage
cargo tarpaulin --out Html

# Check coverage
open tarpaulin-report.html
```

### Golden report tests (refactoring safety net)

`run_scenario` is deterministic (virtual time + ordered maps), so its output is
pinned by golden snapshots:

- `crates/roomci-core/tests/golden/` — one `<scenario>.json` per
  `examples/*.yaml`, pinning the full `RunReport` JSON contract.
- `crates/roomci-report/tests/golden/` — Markdown / JUnit XML / timeline NDJSON /
  observability JSON renders for representative scenarios.

These guard the output contract (schema fields, timeline `event_type` / `message`
strings, assertion names / impact levels) while refactoring. Rules:

1. **Behavior-preserving PRs (most refactors): do not update golden files.** If a
   refactor changes a golden, the behavior changed — split that into its own PR.
2. **Intentional behavior changes: regenerate and review the diff.**
   ```bash
   UPDATE_GOLDEN=1 cargo test -p roomci-core --test golden_reports
   UPDATE_GOLDEN=1 cargo test -p roomci-report --test golden_renders
   ```
   Commit the regenerated files and paste the golden diff into the PR description
   so reviewers can see exactly what changed.
3. A new `examples/*.yaml` requires a new golden — generate it the same way.

## Documentation

- Update README.md for user-facing changes
- Update API docs for code changes
- Include examples in documentation
- Keep comments clear and concise

## Security

If you discover a security vulnerability, please email security@[yourdomain] instead of using the issue tracker.

## License

By contributing to roomci, you agree that your contributions will be licensed under its Apache License 2.0.

## Questions?

- Check existing issues and discussions
- Review the documentation in `/docs`
- Open a discussion for questions

Thank you for contributing! 🎉
