use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, ItemStruct, LitStr, parse2};

pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let path = parse2::<LitStr>(attribute)?;
    let mut item = parse2::<ItemStruct>(item)?;
    let name = &item.ident;
    let guards = take_components(&mut item.attrs, "guard")?;
    let interceptors = take_components(&mut item.attrs, "interceptor")?;
    let middlewares = take_components(&mut item.attrs, "middleware")?;

    Ok(quote! {
        #item

        impl #name {
            /// Returns controller metadata generated from its route declarations.
            pub fn controller_definition() -> ::ironic::ControllerDefinition {
                ::ironic::ControllerDefinition::new::<Self>(
                    #path,
                    Self::provider_definition(),
                )
                .expect("the macro-validated controller path is valid")
                .with_routes(Self::route_definitions())
                #(.guard(#guards))*
                #(.interceptor(#interceptors))*
                #(.middleware(#middlewares))*
            }
        }
    })
}

pub(crate) fn take_components(
    attrs: &mut Vec<Attribute>,
    name: &str,
) -> syn::Result<Vec<syn::Expr>> {
    let mut values = Vec::new();
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        if attr.path().is_ident(name) {
            values.push(attr.parse_args::<syn::Expr>()?);
        } else {
            retained.push(attr);
        }
    }
    *attrs = retained;
    Ok(values)
}
