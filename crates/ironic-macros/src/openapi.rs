use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, LitStr, PathArguments, Type};

pub(crate) fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            input.generics,
            "OpenApiSchema does not yet support generic structs",
        ));
    }
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(
            name,
            "OpenApiSchema can only be derived for structs",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new_spanned(
            name,
            "OpenApiSchema requires named fields",
        ));
    };

    let mut properties = Vec::new();
    let mut required = Vec::new();
    for field in fields.named {
        let ident = field.ident.expect("named fields have identifiers");
        let field_name = serde_rename(&field.attrs)?.unwrap_or_else(|| ident.to_string());
        let ty = field.ty;
        properties.push(quote! {
            properties.insert(
                #field_name.to_owned(),
                <#ty as ::ironic::OpenApiSchema>::openapi_schema(),
            );
        });
        if !is_option(&ty) {
            required.push(field_name);
        }
    }

    Ok(quote! {
        impl ::ironic::OpenApiSchema for #name {
            fn openapi_schema() -> ::ironic::__private::serde_json::Value {
                let mut properties = ::ironic::__private::serde_json::Map::new();
                #(#properties)*
                ::ironic::__private::serde_json::json!({
                    "type": "object",
                    "title": ::std::stringify!(#name),
                    "properties": properties,
                    "required": [#(#required),*]
                })
            }
        }
    })
}

fn is_option(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path.segments.last().is_some_and(|segment| {
        segment.ident == "Option"
            && matches!(segment.arguments, PathArguments::AngleBracketed(ref arguments)
                if arguments.args.iter().any(|argument| matches!(argument, GenericArgument::Type(_))))
    })
}

fn serde_rename(attributes: &[syn::Attribute]) -> syn::Result<Option<String>> {
    let mut rename = None;
    for attribute in attributes {
        if !attribute.path().is_ident("serde") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                rename = Some(meta.value()?.parse::<LitStr>()?.value());
            }
            Ok(())
        })?;
    }
    Ok(rename)
}
