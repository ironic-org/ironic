# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Report security issues to **security@ironic.rs**.

You should receive a response within 48 hours. If not, follow up to ensure
receipt. We will keep you informed of the progress toward a fix and release.

### What to include

- Type of vulnerability
- Steps to reproduce
- Affected versions
- Potential impact
- Suggested fix (if any)

## Disclosure Policy

We follow coordinated disclosure:

1. Receive and confirm the report
2. Work on a fix
3. Release a patch
4. Publicly acknowledge the report (with your consent)

## Security Practices

- `cargo audit` runs in CI on every push
- `cargo deny` checks for advisories and license compliance
- `unsafe_code` is forbidden at the workspace level
- Secrets and credentials must never be committed — use environment variables or secret managers
