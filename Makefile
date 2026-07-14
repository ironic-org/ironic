.PHONY: release release-patch release-minor release-major \
        build test fmt clippy docs dev clean help

# ── Release ────────────────────────────────────────────────────────────

release:              ## Release the current version (no bump)
	@./scripts/release.sh

release-patch:        ## Bump patch version and release (0.1.8 → 0.1.9)
	@./scripts/release.sh patch

release-minor:        ## Bump minor version and release (0.1.8 → 0.2.0)
	@./scripts/release.sh minor

release-major:        ## Bump major version and release (0.1.8 → 1.0.0)
	@./scripts/release.sh major

# ── Development ────────────────────────────────────────────────────────

build:                ## Build the project
	cargo build

test:                 ## Run all tests
	cargo test

test-all:             ## Run all tests with all features
	cargo test --all-features

fmt:                  ## Format all code
	cargo fmt --all

fmt-check:            ## Check formatting
	cargo fmt --all -- --check

clippy:               ## Run clippy with all features
	cargo clippy --workspace --all-targets --all-features -- -D warnings

docs:                 ## Build the documentation site
	@npm --prefix docs run build

dev:                  ## Start dev server with hot reload
	cargo run -- dev

# ── Utilities ──────────────────────────────────────────────────────────

clean:                ## Clean build artifacts
	cargo clean

help:                 ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-18s\033[0m %s\n", $$1, $$2}'
