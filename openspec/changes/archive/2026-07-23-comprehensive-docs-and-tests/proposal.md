## Why

The Ironic framework's crate APIs lack comprehensive doc comments (`///` docs), making IDE hover/autocomplete unhelpful for users. Test coverage is inconsistent — some crates have extensive tests while others have minimal or none. As the codebase grows (22 crates, 458+ public declarations), missing documentation and tests create a steep learning curve for new users, increase bug-fix time, and hurt the project's professional appearance. Adding complete doc comments and test coverage now prevents this debt from compounding.

## What Changes

- Add `///` doc comments to every public function, struct, enum, trait, type alias, const, and macro in all 22 crates — covering purpose, arguments, return values, panics, errors, and examples where applicable
- Add `#[cfg(test)]` modules (or extend existing ones) to achieve meaningful coverage for every public function across all crates
- Reorganize code within files for consistent structure: doc comments → item → impl blocks → tests
- Add `#![deny(missing_docs)]` to each crate's `lib.rs` (behind appropriate feature gates for proc-macro crates) to enforce completeness going forward
- Ensure all examples in doc comments compile and are tested via `#[doc = "..."]` doctests where practical

## Capabilities

### New Capabilities

This change does not introduce new user-facing capabilities. It improves code quality across all existing capabilities.

### Modified Capabilities

- `developer-experience`: Enhanced with comprehensive doc comments on all public APIs, making IDE integrations and API discovery significantly better
- `testing-utilities`: Extended with new test patterns and coverage requirements across all crates

## Impact

- **All 22 crates** in `crates/` — every `.rs` source file will gain doc comments and/or test modules
- **112 source files** and **458+ public declarations** will be touched
- No runtime behavior changes, no dependency changes, no breaking API changes
- Build times may increase slightly from additional tests
- `#![deny(missing_docs)]` will be added to each crate — existing undocumented items must be resolved before this lint can pass
