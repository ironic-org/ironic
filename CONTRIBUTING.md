# Contributing to Ironic

Thanks for your interest in contributing! Ironic is a community-driven open-source project, and every contribution — whether a bug report, feature request, documentation fix, or pull request — is valued.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Finding Work](#finding-work)
- [Making Changes](#making-changes)
- [Commit Style](#commit-style)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)
- [RFC Process](#rfc-process)
- [Security](#security)
- [Getting Help](#getting-help)

## Code of Conduct

All contributors must follow our [Code of Conduct](CODE_OF_CONDUCT.md). Be respectful, constructive, and inclusive.

## Getting Started

1. **Read the docs** at [docs.rs/ironic](https://docs.rs/ironic) or the [local docs site](./docs/) to understand the framework architecture.
2. **Check existing issues** — look for [`good first issue`](https://github.com/ironic-org/ironic/labels/good%20first%20issue) and [`help wanted`](https://github.com/ironic-org/ironic/labels/help%20wanted) labels.
3. **Discuss first** — for significant changes, open an issue or start a discussion before writing code.

## Development Setup

### Prerequisites

- Rust 1.85+ (see `rust-toolchain.toml`)
- Node.js 20+ (for docs site)

### Build and Test

```bash
# Clone and build
git clone https://github.com/ironic-org/ironic.git
cd ironic
cargo build --workspace

# Run tests
cargo test --workspace --all-features

# Run lints
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run security audit
cargo audit
```

### Docs Site

```bash
cd docs
npm install
npm run dev     # local dev server at :3002
npm run build   # production build
```

## Finding Work

- [`good first issue`](https://github.com/ironic-org/ironic/labels/good%20first%20issue) — small, well-scoped tasks for new contributors
- [`help wanted`](https://github.com/ironic-org/ironic/labels/help%20wanted) — contributions needed
- [`RFC needed`](https://github.com/ironic-org/ironic/labels/RFC%20needed) — changes that require an RFC first
- No label? Comment on the issue to ask questions or express interest

## Making Changes

1. Fork the repo and create a branch from `main`:

   ```bash
   git checkout -b feat/your-feature-name
   ```

2. Make your changes. Follow the existing code style — the project uses workspace-level lints enforced by Clippy.
3. Add or update tests. Every new feature should include tests; every bug fix should include a regression test.
4. Update documentation if your change affects public APIs.
5. Ensure CI passes locally:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   ```

## Commit Style

We use conventional commits. Examples:

```
feat(di): add singleton provider resolution
fix(cli): preserve module formatting during registration
docs(modules): add import and export examples
test(core): cover circular dependency detection
refactor(http): extract shared status code logic
```

Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`, `style`

## Pull Request Process

1. Ensure your PR description clearly explains the problem and solution.
2. Link any related issues (e.g., "Closes #123").
3. Add a changelog entry in `RELEASE_NOTES.md` if applicable.
4. Keep PRs focused — one change per PR. Large PRs are harder to review.
5. CI must pass before merging.
6. You may merge after one approving review from a maintainer.

### PR Title Convention

Same as commit style: `feat(http): add request ID middleware`

## Reporting Issues

### Bug Reports

Open a [bug report](https://github.com/ironic-org/ironic/issues/new?template=bug_report.yml). Include:

- Rust version (`rustc --version`)
- Ironic version
- Minimal reproduction steps
- Expected vs actual behavior
- Relevant logs or errors

### Feature Requests

Open a [feature request](https://github.com/ironic-org/ironic/issues/new?template=feature_request.yml). Include:

- What problem you're trying to solve
- Proposed solution
- Alternatives you've considered
- Example API usage (if applicable)

## RFC Process

Significant architectural changes require an RFC. See existing RFCs in [`rfcs/`](./rfcs/).

1. Copy `rfcs/0000-template.md` to `rfcs/XXXX-your-title.md`
2. Fill in the template sections
3. Submit a PR with the RFC
4. Iterate on feedback
5. A maintainer merges or closes the RFC PR

## Security

Report security vulnerabilities to **security@ironic.rs** or follow the process in [`SECURITY.md`](SECURITY.md). Do not open public issues for security vulnerabilities.

## Getting Help

- Open a [discussion](https://github.com/ironic-org/ironic/discussions)
- Join our [Discord](https://discord.gg/ironic)
- Check [docs.rs/ironic](https://docs.rs/ironic)
