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
    lifecycle_init: Vec<Type>,
    lifecycle_bootstrap: Vec<Type>,
    lifecycle_destroy: Vec<Type>,
    lifecycle_shutdown: Vec<Type>,
    lifecycle_configure: Vec<Type>,
    lifecycle_server_ready: Vec<Type>,
    lifecycle_request_init: Vec<Type>,
    lifecycle_request_destroy: Vec<Type>,
    lifecycle_error: Vec<Type>,
    lifecycle_guard_denied: Vec<Type>,
    lifecycle_before_shutdown: Vec<Type>,
    lifecycle_after_shutdown: Vec<Type>,
    lifecycle_module_load: Vec<Type>,
    lifecycle_module_unload: Vec<Type>,
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
                "lifecycle_init" => args.lifecycle_init = values,
                "lifecycle_bootstrap" => args.lifecycle_bootstrap = values,
                "lifecycle_destroy" => args.lifecycle_destroy = values,
                "lifecycle_shutdown" => args.lifecycle_shutdown = values,
                "lifecycle_configure" => args.lifecycle_configure = values,
                "lifecycle_server_ready" => args.lifecycle_server_ready = values,
                "lifecycle_request_init" => args.lifecycle_request_init = values,
                "lifecycle_request_destroy" => args.lifecycle_request_destroy = values,
                "lifecycle_error" => args.lifecycle_error = values,
                "lifecycle_guard_denied" => args.lifecycle_guard_denied = values,
                "lifecycle_before_shutdown" => args.lifecycle_before_shutdown = values,
                "lifecycle_after_shutdown" => args.lifecycle_after_shutdown = values,
                "lifecycle_module_load" => args.lifecycle_module_load = values,
                "lifecycle_module_unload" => args.lifecycle_module_unload = values,
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "expected lifecycle hook or module field",
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

    macro_rules! lifecycle_calls {
        ($vec:ident, $method:ident) => {
            {
                let mut calls: Vec<TokenStream> = Vec::new();
                for ty in &args.$vec {
                    calls.push(quote! { .lifecycle(::ironic::LifecycleDefinition::builder::<#ty>().$method().build()) });
                }
                calls
            }
        };
    }

    let mut lifecycle_all: Vec<TokenStream> = Vec::new();
    lifecycle_all.extend(lifecycle_calls!(lifecycle_init, module_init));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_bootstrap, application_bootstrap));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_destroy, module_destroy));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_shutdown, application_shutdown));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_configure, module_configure));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_server_ready, server_ready));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_request_init, request_init));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_request_destroy, request_destroy));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_error, on_error));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_guard_denied, guard_denied));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_before_shutdown, before_shutdown));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_after_shutdown, after_shutdown));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_module_load, module_load));
    lifecycle_all.extend(lifecycle_calls!(lifecycle_module_unload, module_unload));

    Ok(quote! {
        impl ::ironic::Module for #name {
            fn definition() -> ::ironic::ModuleDefinition {
                ::ironic::ModuleDefinition::builder::<Self>()
                    #(#imports)*
                    #(#providers)*
                    #(#controllers)*
                    #(#exports)*
                    #global_call
                    #(#lifecycle_all)*
                    .build()
            }
        }
    })
}
