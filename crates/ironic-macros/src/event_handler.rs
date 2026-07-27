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
    transport: Option<String>,
}

impl Default for EventHandlerArgs {
    fn default() -> Self {
        Self {
            capacity: 16,
            auto_register: false,
            transport: None,
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
            } else if ident == "transport" {
                input.parse::<Token![=]>()?;
                let lit: syn::LitStr = input.parse()?;
                args.transport = Some(lit.value());
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

    if let Some(ref transport) = args.transport {
        // Transport-based event handler (cross-process)
        let pattern = transport.clone();
        output.extend(quote! {
            #[doc(hidden)]
            #[allow(non_snake_case, missing_docs)]
            #vis fn #reg_name(
                server: &impl ::ironic::distributed::microservices::MicroserviceServer,
            ) {
                use ::std::sync::Arc;
                let handler: ::ironic::distributed::microservices::EventHandler = Arc::new(
                    move |payload: ::std::vec::Vec<u8>,
                          _ctx: ::ironic::distributed::microservices::MessageContext|
                          -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ::std::result::Result<(), ::ironic::distributed::microservices::TransportError>> + ::std::marker::Send>> {
                        let payload = payload.clone();
                        Box::pin(async move {
                            let event: #event_type = ::serde_json::from_slice(&payload)
                                .map_err(|e| ::ironic::distributed::microservices::TransportError(e.to_string()))?;
                            #handler_fn_name(event).await;
                            ::std::result::Result::Ok(())
                        })
                    }
                );
                server.on_event(#pattern, handler);
            }
        });
    } else {
        // 2. In-process EventBus registration (existing behavior).
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
    }

    // 3. If auto_register, emit a registrar struct + AsyncModuleInit impl.
    if auto_register {
        let registrar_name = syn::Ident::new(
            &format!("__EventHandlerAuto_{handler_fn_name}"),
            handler_fn_name.span(),
        );

        let async_init_body = if args.transport.is_some() {
            quote! {
                let server = container
                    .resolve::<::ironic::distributed::transport_provider::EventServer>()
                    .await
                    .map_err(|e| {
                        ::ironic::LifecycleError::new(
                            format!("EVENT_SERVER_RESOLVE: {e}"),
                        )
                    })?;
                #reg_name(&*server);
            }
        } else {
            quote! {
                let event_bus = container
                    .resolve::<::ironic::services::events::EventBus>()
                    .await
                    .map_err(|e| {
                        ::ironic::LifecycleError::new(
                            format!("EVENT_BUS_RESOLVE: {e}"),
                        )
                    })?;
                #reg_name(&event_bus);
            }
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
