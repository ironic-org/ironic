use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, DeriveInput, Ident, Token, Type, bracketed, parse::Parse, parse::ParseStream, parse2,
};

#[derive(Default)]
struct ModuleArgs {
    imports: Vec<Type>,
    providers: Vec<Type>,
    controllers: Vec<Type>,
    exports: Vec<Type>,
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let content;
            bracketed!(content in input);
            let values = content
                .parse_terminated(Type::parse, Token![,])?
                .into_iter()
                .collect();
            match key.to_string().as_str() {
                "imports" => args.imports = values,
                "providers" => args.providers = values,
                "controllers" => args.controllers = values,
                "exports" => args.exports = values,
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "expected `imports`, `providers`, `controllers`, or `exports`",
                    ));
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(args)
    }
}

pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "`Module` does not support generic module types",
        ));
    }

    let has_global = input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("global"));

    let module_attributes: Vec<&Attribute> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("module"))
        .collect();
    if module_attributes.len() != 1 {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`Module` requires exactly one `#[module(...)]` attribute",
        ));
    }
    let args = module_attributes[0].parse_args::<ModuleArgs>()?;
    let name = &input.ident;
    let imports = args.imports.iter().map(|ty| quote!(.import::<#ty>()));
    let providers = args
        .providers
        .iter()
        .map(|ty| quote!(.provider(<#ty>::provider_definition())));
    let controllers = args
        .controllers
        .iter()
        .map(|ty| quote!(.controller(<#ty>::controller_definition())));
    let exports = args.exports.iter().map(|ty| quote!(.export::<#ty>()));
    let global_call = has_global.then(|| quote!(.global()));

    Ok(quote! {
        impl ::ironic::Module for #name {
            fn definition() -> ::ironic::ModuleDefinition {
                ::ironic::ModuleDefinition::builder::<Self>()
                    #(#imports)*
                    #(#providers)*
                    #(#controllers)*
                    #(#exports)*
                    #global_call
                    .build()
            }
        }
    })
}
