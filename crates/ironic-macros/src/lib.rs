//! Procedural macros for declaring Ironic application metadata.

use proc_macro::TokenStream;

mod controller;
mod injectable;
mod module;
mod openapi;
mod routes;
mod serializable;
mod r#test;
mod ws_gateway;

#[proc_macro_derive(Injectable, attributes(injectable))]
/// Derives a dependency-injection provider definition.
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    injectable::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Module, attributes(module, global))]
/// Derives a static application module definition.
pub fn derive_module(input: TokenStream) -> TokenStream {
    module::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(OpenApiSchema, attributes(serde))]
/// Derives an `OpenAPI` schema for a named-field struct.
pub fn derive_openapi_schema(input: TokenStream) -> TokenStream {
    openapi::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Declares a controller and its path prefix.
pub fn controller(attribute: TokenStream, item: TokenStream) -> TokenStream {
    controller::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Collects route metadata from an inherent controller implementation.
pub fn routes(attribute: TokenStream, item: TokenStream) -> TokenStream {
    routes::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Serializable, attributes(exclude, expose))]
/// Derives a `field_rules()` method from `#[exclude]` and `#[expose(role = "...")]`
/// field attributes.
pub fn derive_serializable(input: TokenStream) -> TokenStream {
    serializable::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

macro_rules! marker_attribute {
    ($($name:ident),+ $(,)?) => {$ (
        #[doc = concat!("Route metadata consumed by [`macro@routes`].")]
        #[proc_macro_attribute]
        pub fn $name(_attribute: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    )+};
}

#[proc_macro_attribute]
/// Declares a WebSocket gateway and its path.
pub fn web_socket_gateway(attribute: TokenStream, item: TokenStream) -> TokenStream {
    ws_gateway::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

marker_attribute!(
    get,
    post,
    put,
    patch,
    delete,
    head,
    options,
    body,
    form,
    query,
    param,
    header,
    decorator,
    pipe,
    subscribe_message,
    guard,
    interceptor,
    middleware,
    cache,
    cron,
    exception,
    interval,
    timeout,
    api,
    resp,
);

/// Wraps an async test function with Ironic's Tokio runtime, removing the
/// need for users to depend on `tokio` or use `#[tokio::test]`.
///
/// # Usage
///
/// ```ignore
/// use ironic::test;
///
/// #[test]
/// async fn my_test() {
///     // test body — no tokio dependency needed
/// }
/// ```
#[proc_macro_attribute]
pub fn r#test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    r#test::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Configures an async entry point with Ironic's Tokio runtime.
#[proc_macro_attribute]
pub fn main(attribute: TokenStream, item: TokenStream) -> TokenStream {
    if !attribute.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[ironic::main]` does not accept arguments",
        )
        .into_compile_error()
        .into();
    }
    let mut function = syn::parse_macro_input!(item as syn::ItemFn);
    if function.sig.asyncness.is_none() {
        let error = syn::Error::new_spanned(
            function.sig.fn_token,
            "`#[ironic::main]` requires an async function",
        )
        .into_compile_error();
        return quote::quote!(#error #function).into();
    }
    if !function.sig.inputs.is_empty() {
        let error = syn::Error::new_spanned(
            &function.sig.inputs,
            "`#[ironic::main]` entry points cannot accept arguments",
        )
        .into_compile_error();
        return quote::quote!(#error #function).into();
    }
    function.sig.asyncness = None;
    let body = function.block;
    function.block = Box::new(syn::parse_quote!({
        ::ironic::__private::block_on(async move #body)
    }));
    quote::quote!(#function).into()
}
