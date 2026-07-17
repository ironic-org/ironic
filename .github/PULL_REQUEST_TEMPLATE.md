## Description

<!-- Describe the problem and solution. Link related issues. -->

Closes #<!-- issue number -->

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing behavior)
- [ ] Documentation update
- [ ] Refactor / Performance improvement
- [ ] Test addition / improvement
- [ ] CI / Build / Dependency change

## Checklist

> **Failure to follow any of these rules is grounds for rejection.**
> All boxes must be checked before a PR can be merged.

### Code Style

- [ ] My code follows existing patterns in the codebase
- [ ] I have NOT added unnecessary comments — code should be self-documenting. Comments are only allowed for non-obvious logic
- [ ] My functions are small and focused on a single responsibility
- [ ] All public APIs I introduced are documented (doc comments on types, methods, and public module exports)

### Testing

- [ ] I have added tests that prove my fix is effective or my feature works
- [ ] New and existing tests pass (`cargo test --workspace --all-features`)

### Quality

- [ ] I have run `cargo fmt --all -- --check` — formatting is correct
- [ ] I have run `cargo clippy --workspace --all-targets --all-features -- -D warnings` — no warnings
- [ ] I have read [CONTRIBUTING.md](../CONTRIBUTING.md)
- [ ] I have updated documentation if needed
- [ ] I have added a changelog entry in `RELEASE_NOTES.md` if applicable

## Additional Context

<!-- Any additional information that would help reviewers -->
