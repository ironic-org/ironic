use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, ItemFn, PatType, Token, Type,
    parse::{Parse, ParseStream},
    parse2,
};

struct EventHandlerArgs {
    capacity: usize,
    auto_register: bool,
}

impl Default for EventHandlerArgs {
    fn default() -> Self {
        Self {
            capacity: 16,
            auto_register: true,
        }
    }
}

impl Parse for EventHandlerArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = EventHandlerArgs::default();
        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            if ident == "capacity" {
                input.parse::<Token![=]>()?;
                let lit: syn::LitInt = input.parse()?;
                args.capacity = lit.base10_parse::<usize>().unwrap_or(16);
            } else if ident == "auto_register" {
                args.auto_register = true;
            } else if ident == "manual_register" {
                args.auto_register = false;
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unsupported event option; use `capacity`, `auto_register`, or `manual_register`",
                ));
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}

/// Extracts the event type from an event handler's sole parameter.
fn extract_params(function: &ItemFn) -> syn::Result<Type> {
    let params: Vec<&FnArg> = function
        .sig
        .inputs
        .iter()
        .filter(|arg| !matches!(arg, FnArg::Receiver(_)))
        .collect();

    if params.is_empty() {
        return Err(syn::Error::new_spanned(
            &function.sig,
            "event requires at least one parameter for the event type",
        ));
    }

    if params.len() != 1 {
        return Err(syn::Error::new_spanned(
            &function.sig.inputs,
            "event handlers accept exactly one event parameter",
        ));
    }
    extract_type_from_arg(params[0])
}

/// Extracts the event type from the first param (strips `Arc<>` wrapper).
fn extract_type_from_arg(arg: &FnArg) -> syn::Result<Type> {
    match arg {
        FnArg::Typed(PatType { ty, .. }) => Ok(strip_arc(ty)),
        FnArg::Receiver(_) => Err(syn::Error::new_spanned(
            arg,
            "event parameter must be a typed parameter",
        )),
    }
}

/// If the type is `Arc<T>`, returns `T`. Otherwise returns the type as-is.
fn strip_arc(ty: &Type) -> Type {
    if let Type::Path(type_path) = ty
        && let Some(last_seg) = type_path.path.segments.last()
        && last_seg.ident == "Arc"
        && let syn::PathArguments::AngleBracketed(args) = &last_seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return inner.clone();
    }
    ty.clone()
}

#[allow(clippy::too_many_lines)]
pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args: EventHandlerArgs = if attribute.is_empty() {
        EventHandlerArgs::default()
    } else {
        parse2(attribute)?
    };
    let function: ItemFn = parse2(item)?;

    let auto_register = args.auto_register;
    let handler_fn_name = &function.sig.ident;
    let reg_name = syn::Ident::new(
        &format!("__event_reg_{handler_fn_name}"),
        handler_fn_name.span(),
    );

    let event_type = extract_params(&function)?;
    let vis = &function.vis;

    let mut output = TokenStream::new();

    // 1. Emit the original function unchanged.
    output.extend(quote! { #function });

    let capacity = args.capacity;
    output.extend(quote! {
            #[doc(hidden)]
            #[allow(non_snake_case, missing_docs)]
            #vis fn #reg_name(
                event_bus: &::ironic::services::events::EventBus,
            ) {
                let event_bus = event_bus.clone();
                ::tokio::spawn(async move {
                    let mut subscription: ::ironic::services::events::EventSubscription<#event_type> =
                        event_bus.subscribe::<#event_type>(#capacity).await;
                    while let ::std::option::Option::Some(event) = subscription.recv().await {
                        #handler_fn_name(event).await;
                    }
                });
            }
    });

    // 3. If auto_register, emit a registrar struct + AsyncModuleInit impl.
    if auto_register {
        let registrar_name = syn::Ident::new(
            &format!("__EventAuto_{handler_fn_name}"),
            handler_fn_name.span(),
        );

        let async_init_body = quote! {
                let event_bus = container
                    .resolve::<::ironic::services::events::EventBus>()
                    .await
                    .map_err(|e| {
                        ::ironic::LifecycleError::new(
                            format!("EVENT_BUS_RESOLVE: {}", e),
                        )
                    })?;
                #reg_name(&event_bus);
        };

        output.extend(quote! {
            #[doc(hidden)]
            #[allow(missing_docs, non_camel_case_types)]
            pub struct #registrar_name;

            impl ::ironic::AsyncModuleInit for #registrar_name {
                fn async_init<'a>(
                    &'a self,
                    container: &'a ::ironic::Container,
                ) -> ::ironic::LifecycleFuture<'a> {
                    Box::pin(async move {
                        #async_init_body
                        ::std::result::Result::Ok(())
                    })
                }
            }
        });
    }

    Ok(output)
}
