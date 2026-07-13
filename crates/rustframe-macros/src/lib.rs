//! Procedural macros for declaring `RustFrame` application metadata.

use proc_macro::TokenStream;

mod controller;
mod injectable;
mod module;
mod routes;

#[proc_macro_derive(Injectable, attributes(injectable))]
/// Derives a dependency-injection provider definition.
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    injectable::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Module, attributes(module))]
/// Derives a static application module definition.
pub fn derive_module(input: TokenStream) -> TokenStream {
    module::expand(input.into())
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

macro_rules! marker_attribute {
    ($($name:ident),+ $(,)?) => {$ (
        #[doc = concat!("Route metadata consumed by [`macro@routes`].")]
        #[proc_macro_attribute]
        pub fn $name(_attribute: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    )+};
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
    query,
    param,
    header,
    use_guard,
    use_interceptor,
);

/// Configures an async entry point with `RustFrame`'s Tokio runtime.
#[proc_macro_attribute]
pub fn main(attribute: TokenStream, item: TokenStream) -> TokenStream {
    if !attribute.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[rustframe::main]` does not accept arguments",
        )
        .into_compile_error()
        .into();
    }
    let mut function = syn::parse_macro_input!(item as syn::ItemFn);
    if function.sig.asyncness.is_none() {
        let error = syn::Error::new_spanned(
            function.sig.fn_token,
            "`#[rustframe::main]` requires an async function",
        )
        .into_compile_error();
        return quote::quote!(#error #function).into();
    }
    if !function.sig.inputs.is_empty() {
        let error = syn::Error::new_spanned(
            &function.sig.inputs,
            "`#[rustframe::main]` entry points cannot accept arguments",
        )
        .into_compile_error();
        return quote::quote!(#error #function).into();
    }
    function.sig.asyncness = None;
    let body = function.block;
    function.block = Box::new(syn::parse_quote!({
        ::rustframe::__private::block_on(async move #body)
    }));
    quote::quote!(#function).into()
}
