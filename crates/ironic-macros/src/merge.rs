//! Derives a `merge_into` method for partial-update DTOs.
//!
//! Generates `fn merge_into(self, target: &mut T)` that assigns each
//! `Option<T>` field to the target field when `Some`.
//!
//! # Example
//!
//! ```ignore
//! use ironic::Merge;
//!
//! #[derive(Merge)]
//! pub struct UpdateBlogDto {
//!     pub title: Option<String>,
//!     pub content: Option<String>,
//! }
//!
//! let mut post = BlogPost { title: "Old".into(), content: "Old".into() };
//! UpdateBlogDto { title: Some("New".into()), content: None }.merge_into(&mut post);
//! assert_eq!(post.title, "New");
//! assert_eq!(post.content, "Old");
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse2};

pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    let name = &input.ident;

    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Merge can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(name, "Merge requires named fields"));
    };

    let field_assignments: Vec<TokenStream> = fields
        .named
        .iter()
        .filter_map(|field| {
            let field_ident = field.ident.as_ref()?;
            Some(quote! {
                if let ::std::option::Option::Some(value) = self.#field_ident {
                    target.#field_ident = value;
                }
            })
        })
        .collect();

    Ok(quote! {
        impl #name {
            /// Applies `Some` values from `self` onto `target`.
            ///
            /// Fields where `self` is `None` are left unchanged in `target`.
            pub fn merge_into(self, target: &mut Self) {
                #(#field_assignments)*
            }
        }
    })
}
