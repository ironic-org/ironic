use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Expr, FnArg, ImplItem, ImplItemFn, ItemImpl, Lit, LitInt, LitStr, Meta, Pat,
    ReturnType, Token, Type, parse::Parse, parse::ParseStream, parse2, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::controller::take_components;

const HTTP_METHODS: &[(&str, &str)] = &[
    ("get", "GET"),
    ("post", "POST"),
    ("put", "PUT"),
    ("patch", "PATCH"),
    ("delete", "DELETE"),
    ("head", "HEAD"),
    ("options", "OPTIONS"),
];

pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attribute.is_empty() {
        return Err(syn::Error::new_spanned(
            attribute,
            "`#[routes]` does not accept arguments",
        ));
    }
    let mut item = parse2::<ItemImpl>(item)?;
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            &item.self_ty,
            "`#[routes]` requires an inherent impl",
        ));
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "`#[routes]` does not support generic impl blocks",
        ));
    }
    let self_ty = item.self_ty.clone();
    let mut definitions = Vec::new();

    for impl_item in &mut item.items {
        let ImplItem::Fn(method) = impl_item else {
            continue;
        };
        let Some((http_method, path)) = take_http_method(&mut method.attrs)? else {
            continue;
        };
        definitions.push(expand_method(&self_ty, method, &http_method, &path)?);
    }

    Ok(quote! {
        #item

        impl #self_ty {
            #[doc(hidden)]
            pub fn route_definitions() -> ::std::vec::Vec<::ironic::RouteDefinition> {
                use ::ironic::OpenApiRouteExt;
                ::std::vec![#(#definitions),*]
            }
        }
    })
}

fn take_http_method(attrs: &mut Vec<Attribute>) -> syn::Result<Option<(syn::Ident, LitStr)>> {
    let mut route = None;
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        let Some(name) = attr.path().get_ident().map(ToString::to_string) else {
            retained.push(attr);
            continue;
        };
        let Some((_, constant)) = HTTP_METHODS.iter().find(|(method, _)| *method == name) else {
            retained.push(attr);
            continue;
        };
        if route.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "a handler may declare only one HTTP method attribute",
            ));
        }
        let path = match &attr.meta {
            Meta::Path(_) => LitStr::new("/", attr.span()),
            _ => attr.parse_args::<LitStr>()?,
        };
        route = Some((syn::Ident::new(constant, attr.span()), path));
    }
    *attrs = retained;
    Ok(route)
}

// ── OpenAPI attribute types ──────────────────────────────────────────

struct OpenApiAttrs {
    summary: Option<String>,
    tags: Vec<String>,
    operation_id: Option<String>,
    security: Vec<String>,
    responses: Vec<ResponseAttr>,
    request_body: Option<RequestBodyAttr>,
}

struct ResponseAttr {
    status: String,
    description: String,
    json_type: Option<Type>,
}

struct RequestBodyAttr {
    json_type: Type,
}

/// Parses `#[api(key = "value", ...)]` attributes (`summary`, `tag`, `operation_id`).
fn take_openapi_fields(attrs: &mut Vec<Attribute>) -> syn::Result<OpenApiAttrs> {
    let mut summary = None;
    let mut tags = Vec::new();
    let mut operation_id = None;
    let mut security = Vec::new();
    let mut retained = Vec::new();

    for attr in attrs.drain(..) {
        if !attr.path().is_ident("api") {
            retained.push(attr);
            continue;
        }
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("summary") => {
                    let s = lit_str_from_expr(&nv.value)?;
                    summary = Some(s.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("tag") => {
                    let s = lit_str_from_expr(&nv.value)?;
                    tags.push(s.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("operation_id") => {
                    let s = lit_str_from_expr(&nv.value)?;
                    operation_id = Some(s.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("security") => {
                    let s = lit_str_from_expr(&nv.value)?;
                    security.push(s.value());
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "expected `summary`, `tag`, `operation_id`, or `security`",
                    ));
                }
            }
        }
    }
    *attrs = retained;

    let responses = take_response_attrs(attrs)?;
    let request_body = take_request_body_attr(attrs)?;

    Ok(OpenApiAttrs {
        summary,
        tags,
        operation_id,
        security,
        responses,
        request_body,
    })
}

/// Parses `#[resp(status, "description")]` or `#[resp(status, "description", json = Type)]`.
fn take_response_attrs(attrs: &mut Vec<Attribute>) -> syn::Result<Vec<ResponseAttr>> {
    let mut responses = Vec::new();
    let mut retained = Vec::new();

    for attr in attrs.drain(..) {
        if !attr.path().is_ident("resp") {
            retained.push(attr);
            continue;
        }
        let args: ResponseArgs = attr.parse_args()?;
        responses.push(ResponseAttr {
            status: args.status.to_string(),
            description: args.description.value(),
            json_type: args.json_type,
        });
    }
    *attrs = retained;
    Ok(responses)
}

struct ResponseArgs {
    status: LitInt,
    description: LitStr,
    json_type: Option<Type>,
}

impl Parse for ResponseArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let status: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let description: LitStr = input.parse()?;
        let mut json_type = None;
        if input.parse::<Token![,]>().is_ok() {
            let key: syn::Ident = input.parse()?;
            if key != "json" {
                return Err(syn::Error::new(key.span(), "expected `json`"));
            }
            input.parse::<Token![=]>()?;
            json_type = Some(input.parse()?);
        }
        Ok(ResponseArgs {
            status,
            description,
            json_type,
        })
    }
}

/// Parses `#[body(json = Type)]`.
fn take_request_body_attr(attrs: &mut Vec<Attribute>) -> syn::Result<Option<RequestBodyAttr>> {
    let mut result = None;
    let mut retained = Vec::new();

    for attr in attrs.drain(..) {
        if !attr.path().is_ident("body") {
            retained.push(attr);
            continue;
        }
        let args: RequestBodyArgs = attr.parse_args()?;
        result = Some(RequestBodyAttr {
            json_type: args.json_type,
        });
    }
    *attrs = retained;
    Ok(result)
}

struct RequestBodyArgs {
    json_type: Type,
}

impl Parse for RequestBodyArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        if key != "json" {
            return Err(syn::Error::new(key.span(), "expected `json`"));
        }
        input.parse::<Token![=]>()?;
        let json_type: Type = input.parse()?;
        Ok(RequestBodyArgs { json_type })
    }
}

fn generate_openapi_call(openapi: &OpenApiAttrs) -> Option<TokenStream> {
    if openapi.summary.is_none()
        && openapi.tags.is_empty()
        && openapi.operation_id.is_none()
        && openapi.security.is_empty()
        && openapi.responses.is_empty()
        && openapi.request_body.is_none()
    {
        return None;
    }

    let mut calls: Vec<TokenStream> = Vec::new();

    if let Some(ref s) = openapi.summary {
        calls.push(quote! { .summary(#s) });
    }
    if let Some(ref id) = openapi.operation_id {
        calls.push(quote! { .operation_id(#id) });
    }
    for tag in &openapi.tags {
        calls.push(quote! { .tag(#tag) });
    }
    for scheme in &openapi.security {
        calls.push(quote! {
            .security(#scheme, Vec::<String>::new())
        });
    }

    if let Some(ref rb) = openapi.request_body {
        let ty = &rb.json_type;
        calls.push(quote! {
            .request_body(::ironic::OpenApiRequestBody::json::<#ty>())
        });
    }

    for resp in &openapi.responses {
        let status_str = &resp.status;
        let desc = &resp.description;
        if let Some(ref json_ty) = resp.json_type {
            calls.push(quote! {
                .response(#status_str, ::ironic::OpenApiResponse::new(#desc).json::<#json_ty>())
            });
        } else {
            calls.push(quote! {
                .response(#status_str, ::ironic::OpenApiResponse::new(#desc))
            });
        }
    }

    Some(quote! {
        .openapi(::ironic::OpenApiOperation::new() #(#calls)*)
    })
}

// ── Core expand_method ───────────────────────────────────────────────

fn expand_method(
    self_ty: &Type,
    method: &mut ImplItemFn,
    http_method: &syn::Ident,
    path: &LitStr,
) -> syn::Result<TokenStream> {
    if method.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            method.sig.fn_token,
            "route handlers must be async",
        ));
    }
    if !method.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &method.sig.generics,
            "route handlers cannot be generic",
        ));
    }
    if matches!(method.sig.output, ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &method.sig,
            "route handlers must return `Result<_, HttpError>`",
        ));
    }

    let Some(FnArg::Receiver(receiver)) = method.sig.inputs.first() else {
        return Err(syn::Error::new_spanned(
            &method.sig,
            "route handlers require an `&self` receiver",
        ));
    };
    if receiver.reference.is_none() || receiver.mutability.is_some() {
        return Err(syn::Error::new_spanned(
            receiver,
            "route handlers require an immutable `&self` receiver",
        ));
    }

    let guards = take_components(&mut method.attrs, "guard")?;
    let interceptors = take_components(&mut method.attrs, "interceptor")?;
    let middlewares = take_components(&mut method.attrs, "middleware")?;
    let cache_ttl = take_cache_ttl(&mut method.attrs)?;
    let openapi = take_openapi_fields(&mut method.attrs)?;

    let mut extractors = Vec::new();
    let mut bindings = Vec::new();
    let mut arguments = Vec::new();
    let mut parameter_pipes: Vec<Vec<TokenStream>> = Vec::new();

    for (index, argument) in method.sig.inputs.iter_mut().skip(1).enumerate() {
        let FnArg::Typed(argument) = argument else {
            unreachable!()
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "route parameter patterns must be identifiers",
            ));
        };
        let argument_name = &pattern.ident;
        let argument_type = &argument.ty;
        let (extractor, pipes) = take_extractor(&mut argument.attrs, argument_name, argument_type)?;
        extractors.push(extractor);
        parameter_pipes.push(pipes);
        bindings.push(quote!(let #argument_name = arguments.take::<#argument_type>(#index)?;));
        arguments.push(argument_name);
    }

    let method_name = &method.sig.ident;
    let cache_call = cache_ttl.map(|ttl| {
        quote! { .cache(::ironic::CacheMetadata::new(#ttl)) }
    });
    let openapi_call = generate_openapi_call(&openapi);
    let parameter_calls: Vec<TokenStream> = extractors
        .into_iter()
        .zip(parameter_pipes)
        .map(|(extractor, pipes)| {
            if pipes.is_empty() {
                quote! { .parameter(#extractor) }
            } else {
                quote! { .parameter_with_pipes(#extractor, [#(#pipes),*]) }
            }
        })
        .collect();

    Ok(quote! {
        ::ironic::RouteDefinition::new(
            ::ironic::HttpMethod::#http_method,
            #path,
            ::std::stringify!(#method_name),
            ::ironic::handler_fn(
                |controller: ::std::sync::Arc<#self_ty>, mut arguments| async move {
                    #(#bindings)*
                    controller.#method_name(#(#arguments),*).await
                },
            ),
        )
        .expect("the macro-validated route path is valid")
        #(#parameter_calls)*
        #(.guard(#guards))*
        #(.interceptor(#interceptors))*
        #(.middleware(#middlewares))*
        #cache_call
        #openapi_call
    })
}

fn take_extractor(
    attrs: &mut Vec<Attribute>,
    argument_name: &syn::Ident,
    argument_type: &Type,
) -> syn::Result<(TokenStream, Vec<TokenStream>)> {
    let mut extractor = None;
    let mut pipes = Vec::new();
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        let Some(name) = attr.path().get_ident().map(ToString::to_string) else {
            retained.push(attr);
            continue;
        };
        match name.as_str() {
            "body" => {
                let value = quote!(::ironic::JsonBody::<#argument_type>::new());
                if extractor.replace(value).is_some() {
                    return Err(syn::Error::new_spanned(
                        argument_name,
                        "a route parameter must have exactly one extractor attribute",
                    ));
                }
            }
            "query" => {
                let value = quote!(::ironic::QueryParameters::<#argument_type>::new());
                if extractor.replace(value).is_some() {
                    return Err(syn::Error::new_spanned(
                        argument_name,
                        "a route parameter must have exactly one extractor attribute",
                    ));
                }
            }
            "param" => {
                let name = optional_name(&attr, argument_name)?;
                let value = quote!(::ironic::PathParameter::<#argument_type>::new(#name));
                if extractor.replace(value).is_some() {
                    return Err(syn::Error::new_spanned(
                        argument_name,
                        "a route parameter must have exactly one extractor attribute",
                    ));
                }
            }
            "header" => {
                let name = optional_name(&attr, argument_name)?;
                let value = quote!(::ironic::HeaderParameter::<#argument_type>::new(#name));
                if extractor.replace(value).is_some() {
                    return Err(syn::Error::new_spanned(
                        argument_name,
                        "a route parameter must have exactly one extractor attribute",
                    ));
                }
            }
            "decorator" => {
                let extractor_type: Type = attr.parse_args()?;
                let value = quote!(#extractor_type::new());
                if extractor.replace(value).is_some() {
                    return Err(syn::Error::new_spanned(
                        argument_name,
                        "a route parameter must have exactly one extractor attribute",
                    ));
                }
            }
            "pipe" => {
                let pipe_fn: Expr = attr.parse_args()?;
                pipes.push(quote!(#pipe_fn()));
            }
            _ => {
                retained.push(attr);
            }
        }
    }
    *attrs = retained;
    let extractor = extractor.ok_or_else(|| {
        syn::Error::new_spanned(
            argument_name,
            "route parameters require one of `#[body]`, `#[query]`, `#[param]`, `#[header]`, or `#[decorator(ExtractorType)]`",
        )
    })?;
    Ok((extractor, pipes))
}

fn optional_name(attr: &Attribute, argument_name: &syn::Ident) -> syn::Result<LitStr> {
    match &attr.meta {
        Meta::Path(_) => Ok(LitStr::new(
            &argument_name.to_string(),
            argument_name.span(),
        )),
        _ => attr.parse_args::<LitStr>(),
    }
}

struct CacheArgs {
    ttl_secs: u64,
}

impl Parse for CacheArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        if key != "ttl_secs" {
            return Err(syn::Error::new(key.span(), "expected `ttl_secs`"));
        }
        input.parse::<syn::Token![=]>()?;
        let value: LitInt = input.parse()?;
        Ok(CacheArgs {
            ttl_secs: value.base10_parse()?,
        })
    }
}

fn lit_str_from_expr(expr: &Expr) -> syn::Result<LitStr> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => Ok(s.clone()),
            _ => Err(syn::Error::new_spanned(expr, "expected a string literal")),
        },
        _ => Err(syn::Error::new_spanned(expr, "expected a string literal")),
    }
}

fn take_cache_ttl(attrs: &mut Vec<Attribute>) -> syn::Result<Option<u64>> {
    let mut ttl = None;
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        if attr.path().is_ident("cache") {
            let args: CacheArgs = attr.parse_args()?;
            ttl = Some(args.ttl_secs);
        } else {
            retained.push(attr);
        }
    }
    *attrs = retained;
    Ok(ttl)
}
