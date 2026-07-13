# `RustFrame` CLI

The `rustframe` binary scaffolds applications, delegates standard workflows to Cargo, and generates
framework source without defining runtime behavior.

```text
rustframe new my-api
rustframe start -- --release
rustframe build -- --all-features
rustframe test -- --all-features
rustframe generate module users
rustframe generate controller users
rustframe generate service users
rustframe generate resource products
rustframe doctor
```

`generate` can be shortened to `g`; generator aliases include `mo`, `co`, `s`, and `res`.

Generators normalize names, refuse to overwrite files with divergent content, and produce no
duplicate declarations when repeated. Rust module declarations and `AppModule` imports are updated
through parsed syntax. When a source file is missing, ambiguous, or cannot be parsed safely, the CLI
leaves it unchanged and prints a precise manual registration instruction.
