use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr};

pub(crate) fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "Serializable can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "Serializable requires named fields",
        ));
    };

    let mut rules = Vec::new();

    for field in fields.named {
        let ident = field.ident.expect("named fields have identifiers");
        let field_name = ident.to_string();
        let mut has_exclude = false;
        let mut expose_role: Option<String> = None;

        for attr in &field.attrs {
            if attr.path().is_ident("exclude") {
                has_exclude = true;
            } else if attr.path().is_ident("expose") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("role") {
                        expose_role = Some(meta.value()?.parse::<LitStr>()?.value());
                    }
                    Ok(())
                })?;
            }
        }

        if has_exclude {
            rules.push(quote! {
                rules = rules.exclude(#field_name);
            });
        }
        if let Some(role) = expose_role {
            rules.push(quote! {
                rules = rules.expose(#field_name, #role);
            });
        }
    }

    Ok(quote! {
        impl #name {
            /// Returns serialisation field rules derived from `#[exclude]` /
            /// `#[expose]` attributes.
            pub fn field_rules() -> ::ironic::FieldRules {
                let rules = ::ironic::FieldRules::new();
                #(#rules)*
                rules
            }
        }
    })
}
