use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use quote::ToTokens;
use syn::{Attribute, Expr, Item, Meta, punctuated::Punctuated, token::Comma};

use crate::CliError;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Route {
    method: String,
    path: String,
    controller: String,
    handler: String,
}

pub(crate) fn workspace(root: &Path, output: &mut impl Write) -> Result<(), CliError> {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(CliError::io(
            "open Cargo.toml",
            &manifest,
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no Cargo.toml found — are you in an Ironic project?",
            ),
        ));
    }
    let content = fs::read_to_string(&manifest)
        .map_err(|error| CliError::io("read Cargo.toml", &manifest, error))?;

    let name = content
        .lines()
        .find(|l| l.starts_with("name = "))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("unknown");
    let version = content
        .lines()
        .find(|l| l.starts_with("version = "))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("unknown");
    let edition = content
        .lines()
        .find(|l| l.starts_with("edition = "))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("unknown");

    let modules = list_modules(&root.join("src/modules"))
        .map_err(|e| CliError::io("read modules", root, e))?;

    writeln!(output, "Project: {name}").map_err(|e| CliError::io("write", root, e))?;
    writeln!(output, "Version: {version}").map_err(|e| CliError::io("write", root, e))?;
    writeln!(output, "Edition: {edition}").map_err(|e| CliError::io("write", root, e))?;
    writeln!(output, "Modules: {modules}").map_err(|e| CliError::io("write", root, e))?;
    Ok(())
}

fn list_modules(dir: &Path) -> Result<String, std::io::Error> {
    if !dir.is_dir() {
        return Ok("none".into());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str()
        {
            names.push(name.to_owned());
        }
    }
    names.sort();
    Ok(if names.is_empty() {
        "none".into()
    } else {
        names.join(", ")
    })
}

pub(crate) fn routes(root: &Path, output: &mut impl Write) -> Result<(), CliError> {
    let files = rust_files(root)?;
    let parsed = parse_files(&files)?;
    let mut prefixes = HashMap::new();
    for (_, source) in &parsed {
        for item in &source.items {
            if let Item::Struct(item) = item
                && let Some(prefix) = attribute(&item.attrs, "controller").and_then(string_argument)
            {
                prefixes.insert(item.ident.to_string(), prefix);
            }
        }
    }

    let mut routes = Vec::new();
    for (_, source) in &parsed {
        for item in &source.items {
            let Item::Impl(item) = item else { continue };
            if attribute(&item.attrs, "routes").is_none() {
                continue;
            }
            let controller = item.self_ty.to_token_stream().to_string().replace(' ', "");
            let prefix = prefixes.get(&controller).map_or("", String::as_str);
            for member in &item.items {
                let syn::ImplItem::Fn(function) = member else {
                    continue;
                };
                for (name, method) in [
                    ("get", "GET"),
                    ("post", "POST"),
                    ("put", "PUT"),
                    ("patch", "PATCH"),
                    ("delete", "DELETE"),
                    ("head", "HEAD"),
                    ("options", "OPTIONS"),
                ] {
                    if let Some(route) = attribute(&function.attrs, name) {
                        let suffix = string_argument(route).unwrap_or_else(|| "/".into());
                        routes.push(Route {
                            method: method.into(),
                            path: join_paths(prefix, &suffix),
                            controller: controller.clone(),
                            handler: function.sig.ident.to_string(),
                        });
                    }
                }
            }
        }
    }
    routes.sort();
    routes.dedup();
    writeln!(output, "METHOD  PATH  HANDLER")
        .map_err(|error| CliError::io("write route output", root, error))?;
    for route in routes {
        writeln!(
            output,
            "{:<7} {}  {}::{}",
            route.method, route.path, route.controller, route.handler
        )
        .map_err(|error| CliError::io("write route output", root, error))?;
    }
    Ok(())
}

pub(crate) fn graph(root: &Path, output: &mut impl Write) -> Result<(), CliError> {
    let files = rust_files(root)?;
    let parsed = parse_files(&files)?;
    let mut edges = Vec::new();
    for (_, source) in &parsed {
        for item in &source.items {
            let Item::Struct(item) = item else { continue };
            let name = item.ident.to_string();
            if let Some(module) = attribute(&item.attrs, "module") {
                for (kind, dependency) in module_entries(module) {
                    edges.push((name.clone(), dependency, kind));
                }
            }
            if derives(&item.attrs, "Injectable") {
                for field in &item.fields {
                    edges.push((
                        name.clone(),
                        field.ty.to_token_stream().to_string().replace(' ', ""),
                        "depends_on",
                    ));
                }
            }
        }
    }
    edges.sort();
    edges.dedup();
    writeln!(output, "digraph ironic {{")
        .map_err(|error| CliError::io("write graph output", root, error))?;
    for (from, to, kind) in edges {
        writeln!(output, "  \"{from}\" -> \"{to}\" [label=\"{kind}\"];")
            .map_err(|error| CliError::io("write graph output", root, error))?;
    }
    writeln!(output, "}}").map_err(|error| CliError::io("write graph output", root, error))?;
    Ok(())
}

fn rust_files(root: &Path) -> Result<Vec<PathBuf>, CliError> {
    let source = if root.join("src").is_dir() {
        root.join("src")
    } else {
        root.to_path_buf()
    };
    let mut files = Vec::new();
    collect_rust_files(&source, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), CliError> {
    let entries = fs::read_dir(directory)
        .map_err(|error| CliError::io("read source directory", directory, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| CliError::io("read source entry", directory, error))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_files(files: &[PathBuf]) -> Result<Vec<(PathBuf, syn::File)>, CliError> {
    files
        .iter()
        .map(|path| {
            let content = fs::read_to_string(path)
                .map_err(|error| CliError::io("read Rust source", path, error))?;
            let parsed = syn::parse_file(&content).map_err(|error| CliError::SourceParse {
                path: path.clone(),
                message: error.to_string(),
            })?;
            Ok((path.clone(), parsed))
        })
        .collect()
}

fn attribute<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    attributes
        .iter()
        .find(|attribute| attribute.path().is_ident(name))
}

fn string_argument(attribute: &Attribute) -> Option<String> {
    attribute
        .parse_args::<syn::LitStr>()
        .ok()
        .map(|value| value.value())
}

fn derives(attributes: &[Attribute], target: &str) -> bool {
    attribute(attributes, "derive").is_some_and(|derive| {
        derive
            .parse_args_with(Punctuated::<syn::Path, Comma>::parse_terminated)
            .is_ok_and(|paths| paths.iter().any(|path| path.is_ident(target)))
    })
}

fn module_entries(attribute: &Attribute) -> Vec<(&'static str, String)> {
    let Ok(entries) = attribute.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) else {
        return Vec::new();
    };
    let mut output = Vec::new();
    for entry in entries {
        let Meta::NameValue(entry) = entry else {
            continue;
        };
        let Some(kind) = entry.path.get_ident().map(ToString::to_string) else {
            continue;
        };
        let Expr::Array(values) = entry.value else {
            continue;
        };
        let label = match kind.as_str() {
            "imports" => "imports",
            "providers" => "provides",
            "controllers" => "controls",
            "exports" => "exports",
            _ => continue,
        };
        for value in values.elems {
            output.push((label, value.to_token_stream().to_string().replace(' ', "")));
        }
    }
    output
}

/// Joins a route prefix and suffix, normalising slashes.
fn join_paths(prefix: &str, suffix: &str) -> String {
    let joined = format!(
        "{}/{}",
        prefix.trim_end_matches('/'),
        suffix.trim_start_matches('/')
    );
    if joined.is_empty() || joined == "/" {
        "/".into()
    } else if joined.starts_with('/') {
        joined
    } else {
        format!("/{joined}")
    }
}

#[cfg(test)]
mod tests {
    use syn::{Attribute, parse_quote};

    use crate::CliError;

    #[test]
    fn join_paths_both_empty() {
        assert_eq!(super::join_paths("", ""), "/");
    }

    #[test]
    fn join_paths_root_and_suffix() {
        assert_eq!(super::join_paths("/", "users"), "/users");
    }

    #[test]
    fn join_paths_prefix_and_root() {
        assert_eq!(super::join_paths("/api", "/"), "/api/");
    }

    #[test]
    fn join_paths_normalises_double_slashes() {
        assert_eq!(super::join_paths("/api/", "/users"), "/api/users");
    }

    #[test]
    fn join_paths_prefix_only() {
        assert_eq!(super::join_paths("/api", ""), "/api/");
    }

    #[test]
    fn join_paths_suffix_only() {
        assert_eq!(super::join_paths("", "users"), "/users");
    }

    #[test]
    fn join_paths_no_leading_slash() {
        assert_eq!(super::join_paths("api", "users"), "/api/users");
    }

    #[test]
    fn join_paths_multiple_segments() {
        assert_eq!(
            super::join_paths("/api/v1", "users/:id"),
            "/api/v1/users/:id"
        );
    }

    #[test]
    fn join_paths_trailing_slashes() {
        assert_eq!(super::join_paths("/api/v1///", "///users"), "/api/v1/users");
    }

    #[test]
    fn list_modules_nonexistent_dir() {
        let result = super::list_modules(std::path::Path::new("/nonexistent/path")).unwrap();
        assert_eq!(result, "none");
    }

    #[test]
    fn list_modules_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = super::list_modules(dir.path()).unwrap();
        assert_eq!(result, "none");
    }

    #[test]
    fn list_modules_with_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("users")).unwrap();
        std::fs::create_dir(dir.path().join("products")).unwrap();
        let result = super::list_modules(dir.path()).unwrap();
        assert_eq!(result, "products, users");
    }

    #[test]
    fn attribute_finds_by_name() {
        let attrs: Vec<Attribute> =
            vec![parse_quote!(#[controller("/api")]), parse_quote!(#[routes])];
        assert!(super::attribute(&attrs, "controller").is_some());
        assert!(super::attribute(&attrs, "routes").is_some());
        assert!(super::attribute(&attrs, "nonexistent").is_none());
    }

    #[test]
    fn string_argument_extracts_string() {
        let attr: Attribute = parse_quote!(#[controller("/api/v1")]);
        let value = super::string_argument(&attr);
        assert_eq!(value.as_deref(), Some("/api/v1"));
    }

    #[test]
    fn string_argument_handles_no_args() {
        let attr: Attribute = parse_quote!(#[routes]);
        assert!(super::string_argument(&attr).is_none());
    }

    #[test]
    fn derives_detects_target_trait() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[derive(Debug, Clone, Injectable)])];
        assert!(super::derives(&attrs, "Injectable"));
        assert!(super::derives(&attrs, "Clone"));
        assert!(!super::derives(&attrs, "Serialize"));
    }

    #[test]
    fn derives_false_without_derive() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[controller("/api")])];
        assert!(!super::derives(&attrs, "Injectable"));
    }

    #[test]
    fn module_entries_parses_imports() {
        let attr: Attribute = parse_quote!(#[module(imports = [UsersModule, ProductsModule])]);
        let entries = super::module_entries(&attr);
        assert!(entries.contains(&("imports", "UsersModule".to_string())));
        assert!(entries.contains(&("imports", "ProductsModule".to_string())));
    }

    #[test]
    fn module_entries_parses_providers_and_controllers() {
        let attr: Attribute = parse_quote!(
            #[module(providers = [UserService, UserRepository], controllers = [UserController])]
        );
        let entries = super::module_entries(&attr);
        assert!(entries.contains(&("provides", "UserService".to_string())));
        assert!(entries.contains(&("provides", "UserRepository".to_string())));
        assert!(entries.contains(&("controls", "UserController".to_string())));
    }

    #[test]
    fn module_entries_returns_empty_for_non_module_attr() {
        let attr: Attribute = parse_quote!(#[derive(Debug)]);
        let entries = super::module_entries(&attr);
        assert!(entries.is_empty());
    }

    #[test]
    fn rust_files_collects_recursively() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/modules")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "").unwrap();
        std::fs::write(dir.path().join("src/mod.rs"), "").unwrap();
        std::fs::write(dir.path().join("src/modules/users.rs"), "").unwrap();
        std::fs::write(dir.path().join("README.md"), "").unwrap();
        let files = super::rust_files(dir.path()).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&dir.path().join("src/main.rs")));
        assert!(files.contains(&dir.path().join("src/mod.rs")));
        assert!(files.contains(&dir.path().join("src/modules/users.rs")));
    }

    #[test]
    fn rust_files_handles_directory_without_src() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("lib.rs"), "").unwrap();
        std::fs::write(dir.path().join("nested.rs"), "").unwrap();
        let files = super::rust_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn parse_files_handles_valid_rust() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        let files = vec![dir.path().join("main.rs")];
        let parsed = super::parse_files(&files).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn parse_files_returns_error_on_invalid_rust() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bad.rs"), "this is not valid rust @@").unwrap();
        let files = vec![dir.path().join("bad.rs")];
        let result = super::parse_files(&files);
        assert!(result.is_err());
        assert!(matches!(result, Err(CliError::SourceParse { .. })));
    }

    #[test]
    fn workspace_requires_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let mut buf = Vec::new();
        let result = super::workspace(dir.path(), &mut buf);
        assert!(result.is_err());
        assert!(matches!(result, Err(CliError::Io { .. })));
    }

    #[test]
    fn workspace_output_with_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            r#"[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();
        let mut buf = Vec::new();
        let result = super::workspace(dir.path(), &mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("my-app"));
        assert!(output.contains("0.1.0"));
        assert!(output.contains("2024"));
    }

    #[test]
    fn routes_returns_empty_for_no_controllers() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "").unwrap();
        let mut buf = Vec::new();
        let result = super::routes(dir.path(), &mut buf);
        assert!(result.is_ok());
    }

    #[test]
    fn graph_returns_empty_digraph_for_no_modules() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/lib.rs"), "").unwrap();
        let mut buf = Vec::new();
        let result = super::graph(dir.path(), &mut buf);
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("digraph ironic {"));
    }
}
