use std::{fs, path::Path};

use quote::quote;
use syn::{Ident, Token, Type, bracketed, parse::Parse, parse::ParseStream};

use crate::CliError;

/// Writes `contents` to `path` if it does not exist (new file) or if the existing content
/// matches exactly (no-op). Returns `true` if the file was written, `false` if unchanged.
///
/// # Errors
///
/// Returns [`CliError::FileConflict`] when the file exists with different content.
/// Returns [`CliError::Io`] for directory creation or file-write failures.
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

/// Writes a module shell file unless one already exists with the expected struct.
///
/// # Errors
///
/// Returns [`CliError::SourceParse`] if the existing file cannot be parsed.
/// Returns [`CliError::FileConflict`] if the file exists but lacks the expected struct.
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

/// Ensures that `path` contains the given top-level `declarations`, adding any that are missing.
///
/// Returns `true` if the file was modified.
///
/// # Errors
///
/// Returns [`CliError::SourceParse`] if the file contents cannot be parsed or a declaration
/// cannot be parsed as a `syn::Item`.
/// Returns [`CliError::Io`] if directory creation or file writing fails.
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

/// Adds `import` to the `imports = [...]` list of the first `#[module(...)]` attribute
/// found in `path`, unless it is already present. Returns `true` if the file was modified.
///
/// # Errors
///
/// Returns [`CliError::SourceParse`] when the file cannot be parsed, no `#[module(...)]`
/// attribute is found, or multiple module declarations exist.
/// Returns [`CliError::Io`] if the file read or write fails.
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

#[cfg(test)]
mod tests {
    use crate::CliError;

    #[test]
    fn write_generated_creates_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new_file.rs");
        let result = super::write_generated(&path, "pub fn hello() {}").unwrap();
        assert!(result);
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "pub fn hello() {}");
    }

    #[test]
    fn write_generated_noop_on_exact_match() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("same.rs");
        super::write_generated(&path, "content").unwrap();
        let result = super::write_generated(&path, "content").unwrap();
        assert!(!result);
    }

    #[test]
    fn write_generated_conflict_on_different_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("conflict.rs");
        super::write_generated(&path, "content a").unwrap();
        let result = super::write_generated(&path, "content b");
        assert!(matches!(result, Err(CliError::FileConflict { .. })));
    }

    #[test]
    fn write_generated_creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/deep/file.rs");
        let result = super::write_generated(&path, "fn f() {}").unwrap();
        assert!(result);
        assert!(path.exists());
    }

    #[test]
    fn write_module_shell_creates_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mod.rs");
        let result = super::write_module_shell(&path, "MyModule").unwrap();
        assert!(result);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("MyModule"));
    }

    #[test]
    fn write_module_shell_noop_when_struct_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mod.rs");
        super::write_module_shell(&path, "ExistingModule").unwrap();
        // Calling again should pass because the struct is already there
        let result = super::write_module_shell(&path, "ExistingModule").unwrap();
        assert!(!result);
    }

    #[test]
    fn write_module_shell_conflict_on_unexpected_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mod.rs");
        std::fs::write(&path, "fn unrelated() {}").unwrap();
        let result = super::write_module_shell(&path, "MyModule");
        assert!(matches!(result, Err(CliError::FileConflict { .. })));
    }

    #[test]
    fn ensure_items_adds_declarations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lib.rs");
        std::fs::write(&path, "pub fn existing() {}\n").unwrap();
        let result = super::ensure_items(&path, &["pub mod new_module;"]).unwrap();
        assert!(result);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("pub mod new_module;"));
    }

    #[test]
    fn ensure_items_noop_when_declaration_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lib.rs");
        std::fs::write(&path, "pub mod existing;\n").unwrap();
        let result = super::ensure_items(&path, &["pub mod existing;"]).unwrap();
        assert!(!result);
    }

    #[test]
    fn ensure_items_creates_file_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new.rs");
        let result = super::ensure_items(&path, &["pub mod my_mod;"]).unwrap();
        assert!(result);
        assert!(path.exists());
    }

    #[test]
    fn ensure_module_import_adds_to_module_attr() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.rs");
        std::fs::write(
            &path,
            r#"
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [ExistingModule])]
struct AppModule;
"#,
        )
        .unwrap();
        let result = super::ensure_module_import(&path, "crate::modules::new::NewModule").unwrap();
        assert!(result);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("crate::modules::new::NewModule"));
    }

    #[test]
    fn ensure_module_import_noop_when_already_present() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.rs");
        std::fs::write(
            &path,
            r#"
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [ExistingModule])]
struct AppModule;
"#,
        )
        .unwrap();
        let result = super::ensure_module_import(&path, "ExistingModule").unwrap();
        assert!(!result);
    }

    #[test]
    fn ensure_module_import_errors_on_no_module_attr() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no_module.rs");
        std::fs::write(
            &path,
            r#"struct NoModuleHere;"#,
        )
        .unwrap();
        let result = super::ensure_module_import(&path, "Something");
        assert!(matches!(result, Err(CliError::SourceParse { .. })));
    }

    #[test]
    fn module_metadata_parse_full() {
        use syn::parse2;
        let meta: super::ModuleMetadata = parse2(quote::quote!(
            imports = [UsersModule, ProductsModule],
            providers = [UserService],
            controllers = [UserController],
            exports = [UsersModule],
        ))
        .unwrap();
        assert_eq!(meta.imports.len(), 2);
        assert_eq!(meta.providers.len(), 1);
        assert_eq!(meta.controllers.len(), 1);
        assert_eq!(meta.exports.len(), 1);
    }

    #[test]
    fn module_metadata_parse_empty() {
        use syn::parse2;
        let meta: super::ModuleMetadata = parse2(quote::quote!()).unwrap();
        assert!(meta.imports.is_empty());
        assert!(meta.providers.is_empty());
        assert!(meta.controllers.is_empty());
        assert!(meta.exports.is_empty());
    }

    #[test]
    fn module_metadata_rejects_unknown_key() {
        use syn::parse2;
        let result = parse2::<super::ModuleMetadata>(quote::quote!(unknown = [X]));
        assert!(result.is_err());
    }

    #[test]
    fn module_metadata_round_trip() {
        use syn::parse2;
        let meta: super::ModuleMetadata = parse2(quote::quote!(
            imports = [UsersModule],
            providers = [UserService, UserRepository],
            controllers = [UserController],
        ))
        .unwrap();
        assert_eq!(meta.imports.len(), 1);
        assert_eq!(meta.providers.len(), 2);
        assert_eq!(meta.controllers.len(), 1);
        assert!(meta.exports.is_empty());
    }
}
