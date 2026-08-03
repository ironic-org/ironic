#![allow(clippy::type_complexity)]
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, PatType, Type,
    parse::{Parse, ParseStream},
    parse2,
};

struct MessageHandlerArgs {
    pattern: String,
}

impl Parse for MessageHandlerArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lit: syn::LitStr = input.parse()?;
        Ok(Self {
            pattern: lit.value(),
        })
    }
}

pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args: MessageHandlerArgs = parse2(attribute)?;
    let function: ItemFn = parse2(item)?;

    let pattern = args.pattern;
    let handler_fn_name = &function.sig.ident;
    let reg_name = syn::Ident::new(
        &format!("__message_reg_{handler_fn_name}"),
        handler_fn_name.span(),
    );

    let (request_type, response_type) = extract_types(&function)?;
    let vis = &function.vis;

    let mut output = TokenStream::new();
    output.extend(quote! { #function });

    output.extend(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case, missing_docs)]
        #vis fn #reg_name(
            server: &impl ::ironic::distributed::MicroserviceServer,
        ) {
            use ::std::sync::Arc;
            let handler: ::ironic::distributed::MessageHandler = Arc::new(
                move |payload: ::std::vec::Vec<u8>,
                      _ctx: ::ironic::distributed::MessageContext|
                      -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ::std::result::Result<::std::vec::Vec<u8>, ::ironic::distributed::TransportError>> + ::std::marker::Send>> {
                    let payload = payload.clone();
                    Box::pin(async move {
                        let request: #request_type = ::serde_json::from_slice(&payload)
                            .map_err(|e| ::ironic::distributed::TransportError(e.to_string()))?;
                        let response: #response_type = #handler_fn_name(request).await;
                        let response_bytes = ::serde_json::to_vec(&response)
                            .map_err(|e| ::ironic::distributed::TransportError(e.to_string()))?;
                        ::std::result::Result::Ok(response_bytes)
                    })
                }
            );
            server.on_message(#pattern, handler);
        }
    });

    Ok(output)
}

fn extract_types(function: &ItemFn) -> syn::Result<(Type, Type)> {
    let param = function
        .sig
        .inputs
        .iter()
        .find(|arg| !matches!(arg, FnArg::Receiver(_)))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &function.sig,
                "message requires a non-self parameter for the request type",
            )
        })?;

    let request_type = match param {
        FnArg::Typed(PatType { ty, .. }) => ty.as_ref().clone(),
        FnArg::Receiver(_) => unreachable!(),
    };

    let return_type = match &function.sig.output {
        syn::ReturnType::Type(_, ty) => ty.as_ref().clone(),
        _ => {
            return Err(syn::Error::new_spanned(
                &function.sig,
                "message requires a return type",
            ));
        }
    };

    Ok((request_type, return_type))
}
