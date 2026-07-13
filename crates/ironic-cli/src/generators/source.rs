use std::{fs, path::Path};

use quote::quote;
use syn::{Ident, Token, Type, bracketed, parse::Parse, parse::ParseStream};

use crate::CliError;

pub(crate) fn write_generated(path: &Path, contents: &str) -> Result<bool, CliError> {
    if path.exists() {
        let existing =
            fs::read_to_string(path).map_err(|error| CliError::io("read", path, error))?;
        if existing == contents {
            return Ok(false);
        }
        return Err(CliError::FileConflict {
            path: path.to_owned(),
        });
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| CliError::io("create directory", parent, error))?;
    }
    fs::write(path, contents).map_err(|error| CliError::io("write", path, error))?;
    Ok(true)
}

pub(crate) fn write_module_shell(path: &Path, pascal: &str) -> Result<bool, CliError> {
    if !path.exists() {
        return write_generated(path, &super::templates::module(pascal));
    }
    let source = fs::read_to_string(path).map_err(|error| CliError::io("read", path, error))?;
    let file = syn::parse_file(&source).map_err(|error| CliError::SourceParse {
        path: path.to_owned(),
        message: error.to_string(),
    })?;
    let expected = format!("{pascal}Module");
    if file
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Struct(item) if item.ident == expected.as_str()))
    {
        Ok(false)
    } else {
        Err(CliError::FileConflict {
            path: path.to_owned(),
        })
    }
}

pub(crate) fn ensure_items(path: &Path, declarations: &[&str]) -> Result<bool, CliError> {
    let source = if path.exists() {
        fs::read_to_string(path).map_err(|error| CliError::io("read", path, error))?
    } else {
        String::new()
    };
    let mut file = syn::parse_file(&source).map_err(|error| CliError::SourceParse {
        path: path.to_owned(),
        message: error.to_string(),
    })?;
    let mut changed = false;
    for declaration in declarations {
        let item =
            syn::parse_str::<syn::Item>(declaration).map_err(|error| CliError::SourceParse {
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        let canonical = quote::quote!(#item).to_string();
        let exists = file
            .items
            .iter()
            .any(|existing| quote::quote!(#existing).to_string() == canonical);
        if !exists {
            file.items.push(item);
            changed = true;
        }
    }
    if changed {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| CliError::io("create directory", parent, error))?;
        }
        fs::write(path, prettyplease::unparse(&file))
            .map_err(|error| CliError::io("write", path, error))?;
    }
    Ok(changed)
}

#[derive(Default)]
struct ModuleMetadata {
    imports: Vec<Type>,
    providers: Vec<Type>,
    controllers: Vec<Type>,
    exports: Vec<Type>,
}

impl Parse for ModuleMetadata {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut metadata = Self::default();
        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let content;
            bracketed!(content in input);
            let values = content
                .parse_terminated(Type::parse, Token![,])?
                .into_iter()
                .collect();
            match key.to_string().as_str() {
                "imports" => metadata.imports = values,
                "providers" => metadata.providers = values,
                "controllers" => metadata.controllers = values,
                "exports" => metadata.exports = values,
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "unsupported module metadata key",
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(metadata)
    }
}

pub(crate) fn ensure_module_import(path: &Path, import: &str) -> Result<bool, CliError> {
    let source = fs::read_to_string(path).map_err(|error| CliError::io("read", path, error))?;
    let mut file = syn::parse_file(&source).map_err(|error| CliError::SourceParse {
        path: path.to_owned(),
        message: error.to_string(),
    })?;
    let import = syn::parse_str::<Type>(import).map_err(|error| CliError::SourceParse {
        path: path.to_owned(),
        message: error.to_string(),
    })?;
    let mut candidates = file.items.iter_mut().filter_map(|item| {
        let syn::Item::Struct(item) = item else {
            return None;
        };
        item.attrs
            .iter_mut()
            .find(|attribute| attribute.path().is_ident("module"))
    });
    let Some(attribute) = candidates.next() else {
        return Err(CliError::SourceParse {
            path: path.to_owned(),
            message: "no `#[module(...)]` metadata found".to_owned(),
        });
    };
    if candidates.next().is_some() {
        return Err(CliError::SourceParse {
            path: path.to_owned(),
            message: "multiple module declarations are ambiguous".to_owned(),
        });
    }
    let mut metadata =
        attribute
            .parse_args::<ModuleMetadata>()
            .map_err(|error| CliError::SourceParse {
                path: path.to_owned(),
                message: error.to_string(),
            })?;
    let canonical = quote!(#import).to_string();
    if metadata
        .imports
        .iter()
        .any(|existing| quote!(#existing).to_string() == canonical)
    {
        return Ok(false);
    }
    metadata.imports.push(import);
    let imports = metadata.imports;
    let providers = metadata.providers;
    let controllers = metadata.controllers;
    let exports = metadata.exports;
    attribute.meta = syn::parse_quote!(module(
        imports = [#(#imports),*],
        providers = [#(#providers),*],
        controllers = [#(#controllers),*],
        exports = [#(#exports),*],
    ));
    fs::write(path, prettyplease::unparse(&file))
        .map_err(|error| CliError::io("write", path, error))?;
    Ok(true)
}
