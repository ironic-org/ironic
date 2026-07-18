//! Derives `sqlx::FromRow` for a named-field struct.
//!
//! # Supported attributes
//!
//! | Attribute | Effect |
//! |-----------|--------|
//! | `#[sqlx(rename = "column_name")]` | Maps the field to a different column name |
//! | `#[sqlx(json)]` | Deserializes a JSON column via `serde_json::from_value` |
//! | `#[sqlx(default)]` | Uses `Default::default()` when the column is absent / NULL |
//!
//! # Example
//!
//! ```ignore
//! use ironic::FromRow;
//!
//! #[derive(FromRow)]
//! pub struct BlogPost {
//!     pub id: uuid::Uuid,
//!     pub title: String,
//!     #[sqlx(json)]
//!     pub tags: Vec<String>,
//!     #[sqlx(default)]
//!     pub description: Option<String>,
//! }
//! ```
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, parse2};

pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "FromRow does not support generic structs",
        ));
    }

    let name = &input.ident;
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "FromRow can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "FromRow requires named fields",
        ));
    };

    let mut column_gets = Vec::new();
    let mut field_names = Vec::new();

    for field in &fields.named {
        let field_ident = field.ident.as_ref().expect("named field");
        let column_name = sqlx_column_name(&field.attrs).unwrap_or_else(|| field_ident.to_string());
        let ty = &field.ty;
        let has_json = has_sqlx_json(&field.attrs);
        let has_default = has_sqlx_default(&field.attrs);
        let is_option = is_option_type(ty);

        let column_name_lit = LitStr::new(&column_name, proc_macro2::Span::call_site());

        if has_json {
            column_gets.push(quote! {
                let #field_ident: #ty = {
                    let raw_json: ::ironic::__private::serde_json::Value = ::sqlx::Row::try_get(row, #column_name_lit)?;
                    ::ironic::__private::serde_json::from_value::<#ty>(raw_json)
                        .map_err(|e| ::sqlx::Error::ColumnDecode {
                            index: #column_name_lit,
                            source: Box::new(e),
                        })?
                };
            });
        } else if has_default && is_option {
            column_gets.push(quote! {
                let #field_ident: #ty = ::sqlx::Row::try_get(row, #column_name_lit).unwrap_or_default();
            });
        } else {
            column_gets.push(quote! {
                let #field_ident: #ty = ::sqlx::Row::try_get(row, #column_name_lit)?;
            });
        }

        field_names.push(field_ident);
    }

    Ok(quote! {
        impl<'r, R: ::sqlx::Row> ::sqlx::FromRow<'r, R> for #name {
            fn from_row(row: &'r R) -> ::std::result::Result<Self, ::sqlx::Error> {
                #(#column_gets)*
                ::std::result::Result::Ok(Self { #(#field_names),* })
            }
        }
    })
}

fn sqlx_column_name(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("sqlx") {
            continue;
        }
        let mut rename = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                rename = Some(meta.value()?.parse::<LitStr>()?.value());
            }
            Ok(())
        });
        if rename.is_some() {
            return rename;
        }
    }
    None
}

fn has_sqlx_json(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("sqlx")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("json") {
                        Ok(())
                    } else {
                        Err(meta.error("expected `json`"))
                    }
                })
                .is_ok()
    })
}

fn has_sqlx_default(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("sqlx")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        Ok(())
                    } else {
                        Err(meta.error("expected `default`"))
                    }
                })
                .is_ok()
    })
}

fn is_option_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "Option")
}
