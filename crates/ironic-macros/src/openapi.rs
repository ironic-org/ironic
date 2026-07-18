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

        if has_serde_skip(&field.attrs) {
            continue;
        }

        let field_name = serde_rename(&field.attrs)?.unwrap_or_else(|| ident.to_string());
        let ty = &field.ty;

        let extras = collect_garde_constraints(&field.attrs);
        let is_opt = is_option(ty);
        let has_default = has_serde_default(&field.attrs);

        if extras.is_empty() {
            properties.push(quote! {
                properties.insert(
                    #field_name.to_owned(),
                    <#ty as ::ironic::OpenApiSchema>::openapi_schema(),
                );
            });
        } else {
            properties.push(quote! {
                {
                    let mut schema = <#ty as ::ironic::OpenApiSchema>::openapi_schema();
                    #(#extras)*
                    properties.insert(#field_name.to_owned(), schema);
                }
            });
        }

        if !is_opt && !has_default {
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

fn has_serde_skip(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attr| {
        attr.path().is_ident("serde")
            && attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip")
                        || meta.path.is_ident("skip_serializing")
                        || meta.path.is_ident("skip_deserializing")
                    {
                        Ok(())
                    } else {
                        Err(meta.error("expected skip, skip_serializing, or skip_deserializing"))
                    }
                })
                .is_ok()
    })
}

fn has_serde_default(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attr| {
        attr.path().is_ident("serde")
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

fn collect_garde_constraints(attributes: &[syn::Attribute]) -> Vec<proc_macro2::TokenStream> {
    let mut tokens = Vec::new();
    for attr in attributes {
        if !attr.path().is_ident("garde") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("length") {
                let (min_len, max_len) = parse_garde_length(&meta)?;
                if let Some(min) = min_len {
                    tokens.push(quote! {
                        schema["minLength"] = ::ironic::__private::serde_json::json!(#min);
                    });
                }
                if let Some(max) = max_len {
                    tokens.push(quote! {
                        schema["maxLength"] = ::ironic::__private::serde_json::json!(#max);
                    });
                }
            } else if meta.path.is_ident("range") {
                let (min_val, max_val) = parse_garde_range(&meta)?;
                if let Some(min) = min_val {
                    tokens.push(quote! {
                        schema["minimum"] = ::ironic::__private::serde_json::json!(#min);
                    });
                }
                if let Some(max) = max_val {
                    tokens.push(quote! {
                        schema["maximum"] = ::ironic::__private::serde_json::json!(#max);
                    });
                }
            } else if meta.path.is_ident("email") {
                tokens.push(quote! {
                    schema["format"] = ::ironic::__private::serde_json::json!("email");
                });
            } else if meta.path.is_ident("url") {
                tokens.push(quote! {
                    schema["format"] = ::ironic::__private::serde_json::json!("uri");
                });
            } else if meta.path.is_ident("pattern")
                && let Ok(value) = meta.value()
                && let Ok(lit) = value.parse::<LitStr>()
            {
                let pattern = lit.value();
                tokens.push(quote! {
                    schema["pattern"] = ::ironic::__private::serde_json::json!(#pattern);
                });
            }
            Ok(())
        });
    }
    tokens
}

fn parse_garde_length(
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<(Option<i64>, Option<i64>)> {
    let mut min = None;
    let mut max = None;
    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("min") {
            let value = inner.value()?.parse::<syn::LitInt>()?;
            min = Some(value.base10_parse::<i64>()?);
        } else if inner.path.is_ident("max") {
            let value = inner.value()?.parse::<syn::LitInt>()?;
            max = Some(value.base10_parse::<i64>()?);
        }
        Ok(())
    })?;
    Ok((min, max))
}

fn parse_garde_range(
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<(Option<f64>, Option<f64>)> {
    let mut min = None;
    let mut max = None;
    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("min") {
            let value = inner.value()?.parse::<syn::LitFloat>()?;
            min = Some(value.base10_parse::<f64>()?);
        } else if inner.path.is_ident("max") {
            let value = inner.value()?.parse::<syn::LitFloat>()?;
            max = Some(value.base10_parse::<f64>()?);
        }
        Ok(())
    })?;
    Ok((min, max))
}
