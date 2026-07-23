use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, LitStr, Pat, PatType, Type,
    parse::{Parse, ParseStream},
    parse2, Token,
};

struct McpToolArgs {
    name: LitStr,
    description: Option<String>,
}

impl Parse for McpToolArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        let mut description = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let ident = input.parse::<syn::Ident>()?;
            if ident == "description" {
                input.parse::<Token![=]>()?;
                let desc = input.parse::<LitStr>()?;
                description = Some(desc.value());
            }
        }

        Ok(McpToolArgs { name, description })
    }
}

struct ParamInfo {
    name: syn::Ident,
    ty: Type,
    is_optional: bool,
}

fn extract_params(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> Vec<ParamInfo> {
    let mut params = Vec::new();
    for arg in inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg
            && let Pat::Ident(pat_ident) = pat.as_ref()
        {
            let name = pat_ident.ident.clone();
            let param_ty: Type = (**ty).clone();
            let is_optional = is_option_type(&param_ty);
            params.push(ParamInfo {
                name,
                ty: param_ty,
                is_optional,
            });
        }
    }
    params
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        return seg.ident == "Option";
    }
    false
}

fn ty_to_schema_json(ty: &Type) -> TokenStream {
    if let Type::Path(type_path) = ty {
        let last_seg = type_path.path.segments.last();
        match last_seg.map(|s| s.ident.to_string()).as_deref() {
            Some("String") => {
                return quote! { ::serde_json::json!({ "type": "string" }) };
            }
            Some("bool") => {
                return quote! { ::serde_json::json!({ "type": "boolean" }) };
            }
            Some("i32" | "i64" | "u32" | "u64" | "usize" | "isize") => {
                return quote! { ::serde_json::json!({ "type": "integer" }) };
            }
            Some("f32" | "f64") => {
                return quote! { ::serde_json::json!({ "type": "number" }) };
            }
            Some("Vec") => {
                let inner_ty = extract_inner_type(ty);
                let item_schema = ty_to_schema_json(&inner_ty);
                return quote! { ::serde_json::json!({ "type": "array", "items": #item_schema }) };
            }
            Some("Option") => {
                let inner_ty = extract_inner_type(ty);
                return ty_to_schema_json(&inner_ty);
            }
            _ => {}
        }
    }
    quote! { ::serde_json::json!({}) }
}

fn extract_inner_type(ty: &Type) -> Type {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return inner.clone();
    }
    ty.clone()
}

fn build_param_deserialization(params: &[ParamInfo]) -> TokenStream {
    if params.is_empty() {
        return TokenStream::new();
    }

    let desers: Vec<_> = params
        .iter()
        .map(|p| {
            let name = &p.name;
            let ty = &p.ty;
            let name_str = name.to_string();
            if p.is_optional {
                quote! {
                    let #name: #ty = ::serde_json::from_value(
                        params.get(#name_str).cloned().unwrap_or(::serde_json::Value::Null)
                    ).map_err(|e| format!("failed to deserialize '{}': {}", #name_str, e))?;
                }
            } else {
                quote! {
                    let #name: #ty = ::serde_json::from_value(
                        params.get(#name_str).ok_or_else(|| format!("missing required parameter '{}'", #name_str))?.clone()
                    ).map_err(|e| format!("failed to deserialize '{}': {}", #name_str, e))?;
                }
            }
        })
        .collect();

    quote! { #(#desers)* }
}

fn build_schema_object(params: &[ParamInfo]) -> TokenStream {
    if params.is_empty() {
        return quote! { ::serde_json::json!({ "type": "object", "properties": {} }) };
    }

    let prop_defs: Vec<_> = params
        .iter()
        .map(|p| {
            let name = p.name.to_string();
            let schema = ty_to_schema_json(&p.ty);
            quote! { #name => #schema }
        })
        .collect();

    let required: Vec<_> = params
        .iter()
        .filter(|p| !p.is_optional)
        .map(|p| p.name.to_string())
        .collect();

    quote! {
        {
            let mut properties = ::serde_json::Map::new();
            #(
                properties.insert(#prop_defs);
            )*
            ::serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": [#(#required),*]
            })
        }
    }
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args = parse2::<McpToolArgs>(attr)?;
    let func = parse2::<ItemFn>(item)?;

    if func.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            func.sig.fn_token,
            "`#[mcp_tool]` requires an async function",
        ));
    }

    let fn_name = &func.sig.ident;
    let tool_name = args.name.value();
    let description = args.description.unwrap_or_default();

    let params: Vec<ParamInfo> = extract_params(&func.sig.inputs);
    let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
    let schema = build_schema_object(&params);
    let deserialization = build_param_deserialization(&params);

    let generated_fn_name = syn::Ident::new(
        &format!("mcp_tool_{fn_name}"),
        fn_name.span(),
    );

    Ok(quote! {
        #func

        #[doc = concat!("Returns an [`McpTool`] for `", stringify!(#fn_name), "`.")]
        #[allow(non_snake_case, missing_docs)]
        pub fn #generated_fn_name() -> ::ironic::McpTool {
            ::ironic::McpTool::new(
                #tool_name,
                #description,
                #schema,
                ::std::sync::Arc::new(|params: ::serde_json::Value| {
                    ::std::boxed::Box::pin(async move {
                        #deserialization
                        match #fn_name(#(#param_names),*).await {
                            ::std::result::Result::Ok(result) => {
                                ::std::result::Result::Ok(::serde_json::to_value(result)
                                    .map_err(|e| e.to_string())?)
                            }
                            ::std::result::Result::Err(e) => {
                                ::std::result::Result::Err(e.to_string())
                            }
                        }
                    })
                }),
            )
        }
    })
}
