use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct, parse2};

pub(crate) fn expand_resolver(
    _attribute: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let input: ItemStruct = parse2(item)?;
    Ok(quote! { #input })
}

pub(crate) fn expand_query(_attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let function: ItemFn = parse2(item)?;
    Ok(quote! { #function })
}

pub(crate) fn expand_mutation(
    _attribute: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let function: ItemFn = parse2(item)?;
    Ok(quote! { #function })
}

pub(crate) fn expand_subscription(
    _attribute: TokenStream,
    item: TokenStream,
) -> syn::Result<TokenStream> {
    let function: ItemFn = parse2(item)?;
    Ok(quote! { #function })
}
