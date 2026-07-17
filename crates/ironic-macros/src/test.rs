use proc_macro2::TokenStream;
use quote::quote;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attribute.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[ironic::test]` does not accept arguments",
        ));
    }

    let mut function = syn::parse2::<syn::ItemFn>(item)?;

    if function.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            function.sig.fn_token,
            "`#[ironic::test]` requires an async function",
        ));
    }

    let attrs = function.attrs.clone();
    function.sig.asyncness = None;
    let body = function.block.clone();
    *function.block = syn::parse_quote!({
        ::ironic::__private::block_on(async move #body)
    });

    Ok(quote! {
        #(#attrs)*
        #[::core::prelude::v1::test]
        #function
    })
}
