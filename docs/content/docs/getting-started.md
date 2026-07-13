---
title: Getting started
description: Create, run, and test a RustFrame application.
---

# Getting started

RustFrame requires Rust 1.85 or newer.

```bash
cargo install rustframe-cli
rustframe new my-api
cd my-api
cargo run
```

The repository toolchain also pins the matching `rust-analyzer` component so procedural macros are
expanded by a server built for the same compiler. After changing or installing the toolchain,
reload the editor workspace once. `rust-analyzer analysis-stats .` provides a command-line check if
an editor reports that a proc-macro server exited.

The generated application uses `FrameworkApplication`, an explicit root module, and
`AxumAdapter`. Standard Cargo commands always remain available:

```bash
cargo build
cargo test
cargo run
```

## Add a resource

```bash
rustframe generate resource products
cargo test
```

Generation is idempotent and refuses to overwrite divergent files. When an existing source file
cannot be parsed safely, the CLI prints a manual registration instruction instead of modifying it.

For a source-first walkthrough, see [`examples/rest-api`](../../../examples/rest-api/src/main.rs).
