# Ironic CLI

The `ironic` binary scaffolds applications, delegates workflows to Cargo, and generates modules,
controllers, services, and resources.

Install the published CLI and create a project:

```text
cargo install ironic
ironic new my-api
cd my-api
ironic start
```

Use `ironic new .` to generate into the current directory. Its folder name is normalized into the
Cargo package name, and unrelated existing files are preserved.

See the [complete CLI reference](../../docs/content/docs/cli.md) for generated project structure,
Cargo argument forwarding, generator aliases, safe regeneration, and troubleshooting.

```text
ironic start -- --release
ironic build -- --all-features
ironic test -- --all-features
ironic generate module users
ironic generate controller users
ironic generate service users
ironic generate resource products
ironic doctor
```
