use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemStruct, LitStr, parse2};

/// Entrypoint for `#[WebSocketGateway(path)]`.
pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let path = parse2::<LitStr>(attribute)?;
    let item = parse2::<ItemStruct>(item)?;
    let name = &item.ident;

    Ok(quote! {
        #item

        impl #name {
            /// Returns the DI provider definition for this gateway.
            pub fn provider_definition() -> ::ironic::ProviderDefinition {
                ::ironic::ProviderDefinition::constructor(
                    ::ironic::Scope::Singleton,
                    ::std::vec![],
                    |_resolver| async move { ::std::result::Result::Ok(#name) },
                )
            }

            /// Returns the WebSocket gateway registration.
            pub fn gateway_definition() -> ::ironic::WsGatewayDefinition {
                ::ironic::WsGatewayDefinition {
                    path: #path.to_string(),
                    controller: ::ironic::ProviderKey::of::<Self>(),
                    handler_name: ::std::stringify!(#name),
                }
            }
        }
    })
}
