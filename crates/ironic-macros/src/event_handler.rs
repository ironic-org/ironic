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
            auto_register: false,
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
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}

pub(crate) fn expand(attribute: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args: EventHandlerArgs = if attribute.is_empty() {
        EventHandlerArgs::default()
    } else {
        parse2(attribute)?
    };
    let function: ItemFn = parse2(item)?;

    let capacity = args.capacity;
    let auto_register = args.auto_register;
    let handler_fn_name = &function.sig.ident;
    let reg_name = syn::Ident::new(
        &format!("__event_handler_reg_{handler_fn_name}"),
        handler_fn_name.span(),
    );

    let event_type = extract_event_type(&function)?;
    let vis = &function.vis;

    let mut output = TokenStream::new();

    // 1. Emit the original function unchanged.
    output.extend(quote! { #function });

    // 2. Emit the registration function (async now, since subscribe is async).
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
            &format!("__EventHandlerAuto_{handler_fn_name}"),
            handler_fn_name.span(),
        );
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
                        let event_bus = container
                            .resolve::<::ironic::services::events::EventBus>()
                            .await
                            .map_err(|e| {
                                ::ironic::LifecycleError::new(
                                    format!("EVENT_BUS_RESOLVE: {e}"),
                                )
                            })?;
                        #reg_name(&event_bus);
                        ::std::result::Result::Ok(())
                    })
                }
            }
        });
    }

    Ok(output)
}

fn extract_event_type(function: &ItemFn) -> syn::Result<Type> {
    let param = function
        .sig
        .inputs
        .iter()
        .find(|arg| !matches!(arg, FnArg::Receiver(_)))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &function.sig,
                "event_handler requires at least one non-self parameter for the event type",
            )
        })?;

    match param {
        FnArg::Typed(PatType { ty, .. }) => {
            if let Type::Path(type_path) = ty.as_ref() {
                let last_seg =
                    type_path.path.segments.last().ok_or_else(|| {
                        syn::Error::new_spanned(ty, "could not determine event type")
                    })?;
                if last_seg.ident == "Arc"
                    && let syn::PathArguments::AngleBracketed(args) = &last_seg.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    return Ok(inner.clone());
                }
                return Ok(ty.as_ref().clone());
            }
            Ok(ty.as_ref().clone())
        }
        FnArg::Receiver(_) => Err(syn::Error::new_spanned(
            param,
            "event_handler parameter must be a typed parameter",
        )),
    }
}
