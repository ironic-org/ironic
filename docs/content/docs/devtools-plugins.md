---
title: Devtools and plugins
description: Inspect compiled applications and extend modules with safe plugins.
---

# Devtools and plugins

The `devtools` feature captures modules, provider scopes/dependencies, and routes from compiled
runtime state. Mount `ecosystem::devtools::router(snapshot)` under a development-only path. It serves
a small HTML route table and a machine-readable `snapshot.json`.

Never expose the devtools router publicly: type names and application topology are operational
metadata. The UI is read-only and does not resolve providers, execute handlers, or mutate runtime
state.

The `plugins` feature provides a safe, statically linked `Plugin` trait and ordered
`PluginRegistry`. Plugins transform `ModuleDefinitionBuilder`, so their providers, controllers,
imports, exports, and lifecycle hooks pass through the normal graph validator. Duplicate stable
plugin names are rejected. Native dynamic-library loading is intentionally excluded because it
would require an unstable Rust ABI and unsafe loading boundary.
