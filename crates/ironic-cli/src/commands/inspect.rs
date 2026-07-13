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

pub(crate) fn routes(root: &Path, output: &mut impl Write) -> Result<(), CliError> {
    let files = rust_files(root)?;
    let parsed = parse_files(&files)?;
    let mut prefixes = HashMap::new();
    for (_, source) in &parsed {
        for item in &source.items {
            if let Item::Struct(item) = item {
                if let Some(prefix) = attribute(&item.attrs, "controller").and_then(string_argument)
                {
                    prefixes.insert(item.ident.to_string(), prefix);
                }
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
