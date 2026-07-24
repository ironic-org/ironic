use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, ItemStruct};

pub(crate) fn expand_resolver(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input: ItemStruct = parse2(item)?;
    let name = &input.ident;

    Ok(quote! {
        #[derive(::ironic::Injectable)]
        #input

        impl #name {
            fn __graphql_type_name() -> &'static str {
                ::std::stringify!(#name)
            }
        }
    })
}

pub(crate) fn expand_query(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let function: ItemFn = parse2(item)?;
    let name = &function.sig.ident;
    let vis = &function.vis;

    Ok(quote! {
        #vis async fn #name(&self) -> ::async_graphql::Result<()> {
            Ok(())
        }
    })
}

pub(crate) fn expand_mutation(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let function: ItemFn = parse2(item)?;
    let name = &function.sig.ident;
    let vis = &function.vis;

    Ok(quote! {
        #vis async fn #name(&self) -> ::async_graphql::Result<()> {
            Ok(())
        }
    })
}

pub(crate) fn expand_subscription(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let function: ItemFn = parse2(item)?;
    let name = &function.sig.ident;
    let vis = &function.vis;

    Ok(quote! {
        #vis async fn #name(&self) -> ::async_graphql::Result<()> {
            Ok(())
        }
    })
}
