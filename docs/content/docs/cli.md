---
title: CLI reference
description: Install and use the Ironic CLI to create, run, test, and extend applications.
---

# CLI reference

The `ironic` package includes the `ironic` command-line tool. It creates new applications,
delegates build workflows to Cargo, generates framework source files, and checks the local
development environment.

## Install or update

Install the latest published release from crates.io:

```bash
cargo install ironic
```

Update an existing installation:

```bash
cargo install ironic --force
```

Framework contributors can install the CLI from a local checkout:

```bash
cargo install --path .
```

Confirm which executable and version are active:

```bash
which ironic
ironic --version
ironic --help
```

## Create an application

Run `new` from the directory that should contain the project:

```bash
ironic new my-api
cd my-api
ironic start
```

To initialize the directory you are already in, pass `.`. The project name is inferred from the
current directory and normalized for Cargo:

```bash
mkdir test_ironic
cd test_ironic
ironic new .
ironic start
```

Names are normalized for their destination: `My API` becomes the `my-api` directory and Rust-safe
identifiers inside generated source. A name must contain a letter and cannot resolve to a Rust
keyword.

An existing directory may contain unrelated files such as `README.md` or `.git`. Before writing,
the CLI checks every generated path and refuses to overwrite a conflicting `Cargo.toml`,
`ironic.toml`, or source file. A new project contains:

```text
my-api/
├── Cargo.toml
├── ironic.toml
└── src/
    ├── app.rs
    ├── main.rs
    └── modules/
        └── mod.rs
```

`Cargo.toml` declares the single `ironic` dependency. `main.rs` configures `AxumAdapter` and starts
the application on `127.0.0.1:3000`; `app.rs` declares the root application module.

## Run Cargo workflows

The workflow commands execute Cargo in the current directory:

| Ironic command | Executed command |
| --- | --- |
| `ironic start` | `cargo run` |
| `ironic build` | `cargo build` |
| `ironic test` | `cargo test` |

Place extra Cargo arguments after `--`:

```bash
ironic start -- --release
ironic build -- --all-features
ironic test -- --workspace --all-targets
```

If Cargo fails, Ironic returns a non-zero exit status and reports the failed Cargo command.

## Generate source files

Generators run against the current project. The long form is `ironic generate`; `ironic g` is an
alias.

### Complete resource

Generate a module, injectable service, controller, and their registrations together:

```bash
ironic generate resource products
# Short form:
ironic g res products
```

This creates:

```text
src/modules/products/
├── mod.rs
├── products_controller.rs
└── products_service.rs
```

It also registers `products` in `src/modules/mod.rs` and imports `ProductsModule` from
`src/app.rs` when those files can be updated safely.

### Individual generators

```bash
ironic generate module users
ironic generate controller users
ironic generate service users
```

Available aliases are:

| Generator | Alias | Result |
| --- | --- | --- |
| `module` | `mo` | Creates the module directory and module definition. |
| `controller` | `co` | Creates a controller inside the same-named module. |
| `service` | `s` | Creates an injectable service inside the same-named module. |
| `resource` | `res` | Creates and registers the complete vertical slice. |

The controller and service generators print a `manual:` instruction when their generated type
still needs to be added to the module's `controllers` or `providers` list.

## Safe regeneration

Generation is deterministic and safe to run repeatedly:

- `created` means a file was created or safely updated.
- `unchanged` means the existing source already matches the requested result.
- `manual:` means the CLI could not safely make a source-level decision and tells you what to add.
- A conflicting generated file is never silently overwritten.

Commit or review your work before generating a large resource so the resulting changes are easy
to inspect.

## Check the environment

Run the doctor from an application directory:

```bash
ironic doctor
```

It checks:

- the installed Rust compiler;
- the installed Cargo executable;
- whether `Cargo.toml` exists in the current directory;
- whether the manifest contains an Ironic dependency.

`WARN` lines identify missing tools or project configuration without changing any files.

## Troubleshooting

### Command not found

Ensure Cargo's binary directory is on `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then open a new terminal and run `ironic --version`.

### Proc-macro server exited

The repository pins Rust and `rust-analyzer` to compatible versions. From the project directory,
run:

```bash
cargo clean
cargo check
rust-analyzer analysis-stats .
```

Then reload the editor window. Do not combine an editor-provided `rust-analyzer` from one
toolchain with a compiler from an incompatible toolchain.

### Generator refuses to overwrite a file

The file differs from the deterministic template. Keep the existing implementation and apply the
printed manual instruction, or move the file aside, regenerate it, and merge the two versions by
hand.

## Command summary

```text
ironic new <name|.>
ironic start [-- <cargo arguments>...]
ironic build [-- <cargo arguments>...]
ironic test [-- <cargo arguments>...]
ironic generate module <name>
ironic generate controller <name>
ironic generate service <name>
ironic generate resource <name>
ironic doctor
```
