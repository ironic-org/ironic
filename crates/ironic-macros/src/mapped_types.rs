use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr, Token, Type, parse::ParseStream, punctuated::Punctuated};

pub(crate) fn expand_partial(input: DeriveInput) -> syn::Result<TokenStream> {
    let base_type = find_base_type(&input.attrs, "partial")?;
    let name = input.ident;
    Ok(quote! {
        impl ::ironic::OpenApiSchema for #name {
            fn openapi_schema() -> ::ironic::__private::serde_json::Value {
                let base = <#base_type as ::ironic::OpenApiSchema>::openapi_schema();
                let mut schema = base.as_object().cloned()
                    .unwrap_or_default();
                schema.remove("required");
                ::ironic::__private::serde_json::Value::Object(schema)
            }
        }
    })
}

fn pick_omit_impl(
    name: &syn::Ident,
    base_type: &Type,
    fields: &[String],
    include: bool,
) -> TokenStream {
    let field_strs: Vec<String> = fields.to_vec();
    let var_name = if include {
        quote! { allowed }
    } else {
        quote! { excluded }
    };

    quote! {
        impl ::ironic::OpenApiSchema for #name {
            fn openapi_schema() -> ::ironic::__private::serde_json::Value {
                let base = <#base_type as ::ironic::OpenApiSchema>::openapi_schema();
                let base_obj = base.as_object().cloned()
                    .unwrap_or_default();
                let all_properties = base_obj.get("properties")
                    .and_then(|p| p.as_object())
                    .cloned()
                    .unwrap_or_default();
                let #var_name: ::std::collections::HashSet<String> =
                    [#(#field_strs),*].into_iter().map(|s| s.to_string()).collect();
                let mut properties = ::ironic::__private::serde_json::Map::new();
                let mut required: Vec<String> = Vec::new();
                for (key, value) in all_properties {
                    if #var_name.contains(&key) {
                        let is_required = base_obj.get("required")
                            .and_then(|r| r.as_array())
                            .map(|arr| arr.iter().any(|v| v.as_str() == Some(key.as_str())))
                            .unwrap_or(false);
                        if is_required {
                            required.push(key.clone());
                        }
                        properties.insert(key, value);
                    }
                }
                ::ironic::__private::serde_json::json!({
                    "type": "object",
                    "title": ::std::stringify!(#name),
                    "properties": properties,
                    "required": required
                })
            }
        }
    }
}

pub(crate) fn expand_pick(input: DeriveInput) -> syn::Result<TokenStream> {
    let (base_type, fields) = find_type_and_fields(&input.attrs, "pick")?;
    let name = input.ident;
    Ok(pick_omit_impl(&name, &base_type, &fields, true))
}

pub(crate) fn expand_omit(input: DeriveInput) -> syn::Result<TokenStream> {
    let (base_type, fields) = find_type_and_fields(&input.attrs, "omit")?;
    let name = input.ident;
    Ok(pick_omit_impl(&name, &base_type, &fields, false))
}

fn find_base_type(attrs: &[syn::Attribute], name: &str) -> syn::Result<Type> {
    for attr in attrs {
        if attr.path().is_ident(name) {
            return attr.parse_args::<Type>();
        }
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("expected #[{name}(BaseType)] attribute"),
    ))
}

fn find_type_and_fields(attrs: &[syn::Attribute], name: &str) -> syn::Result<(Type, Vec<String>)> {
    for attr in attrs {
        if !attr.path().is_ident(name) {
            continue;
        }
        return attr.parse_args_with(|input: ParseStream<'_>| {
            let base: Type = input.parse()?;
            let _: Token![,] = input.parse()?;
            let mut fields = Vec::new();
            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;
                if ident == "fields" {
                    let _: Token![=] = input.parse()?;
                    let content;
                    syn::bracketed!(content in input);
                    let parsed: Punctuated<LitStr, Token![,]> =
                        content.parse_terminated(syn::parse::Parse::parse, Token![,])?;
                    fields = parsed.into_iter().map(|s| s.value()).collect();
                }
                if !input.is_empty() {
                    let _ = input.parse::<Token![,]>();
                }
            }
            if fields.is_empty() {
                return Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("expected #[{name}(BaseType, fields = [...]])"),
                ));
            }
            Ok((base, fields))
        });
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("expected #[{name}(BaseType, fields = [...]]) attribute"),
    ))
}
