use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, FnArg, ImplItem, ImplItemFn, ItemImpl, LitStr, Meta, Pat, ReturnType, Type, parse2,
    spanned::Spanned,
};

use crate::controller::take_components;

const HTTP_METHODS: &[(&str, &str)] = &[
    ("get", "GET"),
    ("post", "POST"),
    ("put", "PUT"),
    ("patch", "PATCH"),
    ("delete", "DELETE"),
    ("head", "HEAD"),
    ("options", "OPTIONS"),
];

pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attribute.is_empty() {
        return Err(syn::Error::new_spanned(
            attribute,
            "`#[routes]` does not accept arguments",
        ));
    }
    let mut item = parse2::<ItemImpl>(item)?;
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(
            &item.self_ty,
            "`#[routes]` requires an inherent impl",
        ));
    }
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "`#[routes]` does not support generic impl blocks",
        ));
    }
    let self_ty = item.self_ty.clone();
    let mut definitions = Vec::new();

    for impl_item in &mut item.items {
        let ImplItem::Fn(method) = impl_item else {
            continue;
        };
        let Some((http_method, path)) = take_http_method(&mut method.attrs)? else {
            continue;
        };
        definitions.push(expand_method(&self_ty, method, &http_method, &path)?);
    }

    Ok(quote! {
        #item

        impl #self_ty {
            #[doc(hidden)]
            pub fn route_definitions() -> ::std::vec::Vec<::rustframe::RouteDefinition> {
                ::std::vec![#(#definitions),*]
            }
        }
    })
}

fn take_http_method(attrs: &mut Vec<Attribute>) -> syn::Result<Option<(syn::Ident, LitStr)>> {
    let mut route = None;
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        let Some(name) = attr.path().get_ident().map(ToString::to_string) else {
            retained.push(attr);
            continue;
        };
        let Some((_, constant)) = HTTP_METHODS.iter().find(|(method, _)| *method == name) else {
            retained.push(attr);
            continue;
        };
        if route.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "a handler may declare only one HTTP method attribute",
            ));
        }
        let path = match &attr.meta {
            Meta::Path(_) => LitStr::new("/", attr.span()),
            _ => attr.parse_args::<LitStr>()?,
        };
        route = Some((syn::Ident::new(constant, attr.span()), path));
    }
    *attrs = retained;
    Ok(route)
}

fn expand_method(
    self_ty: &Type,
    method: &mut ImplItemFn,
    http_method: &syn::Ident,
    path: &LitStr,
) -> syn::Result<TokenStream> {
    if method.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            method.sig.fn_token,
            "route handlers must be async",
        ));
    }
    if !method.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &method.sig.generics,
            "route handlers cannot be generic",
        ));
    }
    if matches!(method.sig.output, ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &method.sig,
            "route handlers must return `Result<_, HttpError>`",
        ));
    }

    let Some(FnArg::Receiver(receiver)) = method.sig.inputs.first() else {
        return Err(syn::Error::new_spanned(
            &method.sig,
            "route handlers require an `&self` receiver",
        ));
    };
    if receiver.reference.is_none() || receiver.mutability.is_some() {
        return Err(syn::Error::new_spanned(
            receiver,
            "route handlers require an immutable `&self` receiver",
        ));
    }

    let guards = take_components(&mut method.attrs, "use_guard")?;
    let interceptors = take_components(&mut method.attrs, "use_interceptor")?;
    let mut extractors = Vec::new();
    let mut bindings = Vec::new();
    let mut arguments = Vec::new();

    for (index, argument) in method.sig.inputs.iter_mut().skip(1).enumerate() {
        let FnArg::Typed(argument) = argument else {
            unreachable!()
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "route parameter patterns must be identifiers",
            ));
        };
        let argument_name = &pattern.ident;
        let argument_type = &argument.ty;
        let extractor = take_extractor(&mut argument.attrs, argument_name, argument_type)?;
        extractors.push(extractor);
        bindings.push(quote!(let #argument_name = arguments.take::<#argument_type>(#index)?;));
        arguments.push(argument_name);
    }

    let method_name = &method.sig.ident;
    Ok(quote! {
        ::rustframe::RouteDefinition::new(
            ::rustframe::HttpMethod::#http_method,
            #path,
            ::std::stringify!(#method_name),
            ::rustframe::handler_fn(
                |controller: ::std::sync::Arc<#self_ty>, mut arguments| async move {
                    #(#bindings)*
                    controller.#method_name(#(#arguments),*).await
                },
            ),
        )
        .expect("the macro-validated route path is valid")
        #(.parameter(#extractors))*
        #(.guard(#guards))*
        #(.interceptor(#interceptors))*
    })
}

fn take_extractor(
    attrs: &mut Vec<Attribute>,
    argument_name: &syn::Ident,
    argument_type: &Type,
) -> syn::Result<TokenStream> {
    let mut extractor = None;
    let mut retained = Vec::new();
    for attr in attrs.drain(..) {
        let Some(name) = attr.path().get_ident().map(ToString::to_string) else {
            retained.push(attr);
            continue;
        };
        let value = match name.as_str() {
            "body" => quote!(::rustframe::JsonBody::<#argument_type>::new()),
            "query" => quote!(::rustframe::QueryParameters::<#argument_type>::new()),
            "param" => {
                let name = optional_name(&attr, argument_name)?;
                quote!(::rustframe::PathParameter::<#argument_type>::new(#name))
            }
            "header" => {
                let name = optional_name(&attr, argument_name)?;
                quote!(::rustframe::HeaderParameter::<#argument_type>::new(#name))
            }
            _ => {
                retained.push(attr);
                continue;
            }
        };
        if extractor.replace(value).is_some() {
            return Err(syn::Error::new_spanned(
                argument_name,
                "a route parameter must have exactly one extractor attribute",
            ));
        }
    }
    *attrs = retained;
    extractor.ok_or_else(|| {
        syn::Error::new_spanned(
            argument_name,
            "route parameters require one of `#[body]`, `#[query]`, `#[param]`, or `#[header]`",
        )
    })
}

fn optional_name(attr: &Attribute, argument_name: &syn::Ident) -> syn::Result<LitStr> {
    match &attr.meta {
        Meta::Path(_) => Ok(LitStr::new(
            &argument_name.to_string(),
            argument_name.span(),
        )),
        _ => attr.parse_args::<LitStr>(),
    }
}
