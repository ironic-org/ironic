## Context

The Ironic framework has 22 crates with 112 source files and 458+ public declarations. Currently, doc comments are sparse and inconsistent — some public APIs have no documentation at all. Test coverage varies widely across crates. As the framework grows, this makes the API hard to discover via IDE hover/autocomplete and increases maintenance burden.

The crate organization follows a clear pattern (http, auth, config, di, etc.) but individual files lack consistent structure for docs → code → tests.

## Goals / Non-Goals

**Goals:**
- Every public function, struct, enum, trait, type alias, const, and macro across all 22 crates has a `///` doc comment explaining purpose, arguments, return values, panics, errors, and examples
- Every crate gains `#![deny(missing_docs)]` to enforce completeness
- Every public function has at least one test case (unit or integration)
- Consistent file structure: doc comments → code → `#[cfg(test)]` module
- Existing test suites are preserved and extended, not replaced

**Non-Goals:**
- No runtime behavior changes
- No API surface changes (no renames, no type changes, no signature changes)
- No new feature flags or dependencies
- No internal/private API documentation (only public surface)
- No refactoring of implementation logic beyond reordering for consistency

## Decisions

**Decision 1: Inline `#[cfg(test)]` modules at end of each file**
- Rationale: Mirrors existing pattern (22 files already use this). Keeps tests co-located with the code they test. No separate test files needed for unit tests.
- Alternatives considered: Separate `tests/` directory per crate — rejected because existing pattern is inline.

**Decision 2: Use `#![deny(missing_docs)]` in each `lib.rs`, not at workspace level**
- Rationale: Some crates may have legitimate internal-only items. Per-crate allows granular control. Proc-macro crates (`ironic-macros`) will not get this lint since proc-macro internals differ.
- Alternatives considered: Workspace-level deny — too inflexible.

**Decision 3: No doc-generation features (e.g., `rustdoc`-specific)**
- Rationale: Doc comments should be plain and useful at the source level. Fancy rustdoc features (embeds, includes) add complexity without proportional benefit for a framework codebase.

**Decision 4: Doc comments include `# Errors` and `# Panics` sections where applicable**
- Rationale: These are standard Rust doc conventions. They are especially critical in a framework where callers need to handle errors correctly.

**Decision 5: Process crates in dependency order (leaves first)**
- Rationale: Crates with no internal deps (e.g., `ironic-common`, `ironic-di`) can be done independently. Then their downstream consumers. This prevents blocking and allows parallel work.

## Risks / Trade-offs

- **[Risk] ~450 public items to document** → Mitigation: Process crate by crate, starting with small/simple crates first for momentum. Use structured patterns to avoid decision fatigue.
- **[Risk] Some doc comments may become stale** → Mitigation: `#![deny(missing_docs)]` only ensures presence, not accuracy. Pair with code review for new contributions.
- **[Risk] Test additions may reveal latent bugs** → Mitigation: This is a feature, not a bug. Flag and fix as encountered, or document known issues in tests.
- **[Risk] Increased compile time from additional tests** → Mitigation: Acceptable trade-off. Tests only compile during `cargo test`, not in builds.
