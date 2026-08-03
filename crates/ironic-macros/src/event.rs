use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, PatType, Token, Type,
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
            auto_register: true,
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
            } else if ident == "manual_register" {
                args.auto_register = false;
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

/// Info about an injected DI parameter for the event handler.
struct InjectedParam {
    /// Parameter name as written in the function signature.
    name: Ident,
    /// The full type as written (e.g. `Arc<EventClient>`).
    full_type: Type,
    /// The inner type for container resolution (e.g. `EventClient`).
    inner_type: Type,
}

/// Extracts the event type and any injected DI params from the function.
///
/// First non-receiver param is the event type.
/// Subsequent params (e.g. `events: Arc<EventClient>`) become injected dependencies.
fn extract_params(function: &ItemFn) -> syn::Result<(Type, Vec<InjectedParam>)> {
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

    let event_type = extract_type_from_arg(params[0])?;
    let mut injected = Vec::new();
    for arg in &params[1..] {
        injected.push(extract_injected_param(arg)?);
    }

    Ok((event_type, injected))
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

/// Extracts a named injected parameter with its full type and inner type.
fn extract_injected_param(arg: &FnArg) -> syn::Result<InjectedParam> {
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let name = match pat.as_ref() {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        pat,
                        "expected a simple parameter name",
                    ));
                }
            };
            let full_type = ty.as_ref().clone();
            let inner_type = strip_arc(ty);
            Ok(InjectedParam {
                name,
                full_type,
                inner_type,
            })
        }
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

    let (event_type, injected_params) = extract_params(&function)?;
    let vis = &function.vis;

    let mut output = TokenStream::new();

    // 1. Emit the original function unchanged.
    output.extend(quote! { #function });

    // Build the handler call with injected params passed through
    let handler_call = if injected_params.is_empty() {
        quote! { #handler_fn_name(event).await; }
    } else {
        let arg_names: Vec<_> = injected_params.iter().map(|p| &p.name).collect();
        quote! { #handler_fn_name(event, #(#arg_names),*).await; }
    };

    if let Some(ref transport) = args.transport {
        // Transport-based event handler (cross-process)
        let pattern = transport.clone();
        let injected_clones: Vec<TokenStream> = injected_params
            .iter()
            .map(|p| {
                let name = &p.name;
                quote! { let #name = #name.clone(); }
            })
            .collect();
        let injected_sig: Vec<TokenStream> = injected_params
            .iter()
            .map(|p| {
                let name = &p.name;
                let full_type = &p.full_type;
                quote! { #name: #full_type }
            })
            .collect();

        output.extend(quote! {
            #[doc(hidden)]
            #[allow(non_snake_case, missing_docs)]
            #vis fn #reg_name(
                server: &impl ::ironic::distributed::microservices::MicroserviceServer,
                #(#injected_sig),*
            ) {
                use ::std::sync::Arc;
                let handler: ::ironic::distributed::microservices::EventHandler = Arc::new(
                    move |payload: ::std::vec::Vec<u8>,
                          _ctx: ::ironic::distributed::microservices::MessageContext|
                          -> ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ::std::result::Result<(), ::ironic::distributed::microservices::TransportError>> + ::std::marker::Send>> {
                        let payload = payload.clone();
                        #(#injected_clones)*
                        Box::pin(async move {
                            let event: #event_type = ::serde_json::from_slice(&payload)
                                .map_err(|e| ::ironic::distributed::microservices::TransportError(e.to_string()))?;
                            #handler_call
                            ::std::result::Result::Ok(())
                        })
                    }
                );
                server.on_event(#pattern, handler);
            }
        });
    } else {
        // 2. In-process EventBus registration (existing behavior).
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
                        #handler_call
                    }
                });
            }
        });
    }

    // 3. If auto_register, emit a registrar struct + AsyncModuleInit impl.
    if auto_register {
        let registrar_name = syn::Ident::new(
            &format!("__EventAuto_{handler_fn_name}"),
            handler_fn_name.span(),
        );

        let async_init_body = if args.transport.is_some() {
            let injected_resolves: Vec<TokenStream> = injected_params
                .iter()
                .map(|p| {
                    let name = &p.name;
                    let inner = &p.inner_type;
                    quote! {
                        let #name = container
                            .resolve::<#inner>()
                            .await
                            .map_err(|e| {
                                ::ironic::LifecycleError::new(
                                    format!("{}_RESOLVE: {}", stringify!(#name), e),
                                )
                            })?;
                    }
                })
                .collect();
            let injected_args: Vec<&Ident> = injected_params.iter().map(|p| &p.name).collect();

            quote! {
                let server = container
                    .resolve::<::ironic::distributed::transport_provider::EventServer>()
                    .await
                    .map_err(|e| {
                        ::ironic::LifecycleError::new(
                            format!("EVENT_SERVER_RESOLVE: {}", e),
                        )
                    })?;
                #(#injected_resolves)*
                #reg_name(&*server, #(#injected_args),*);
            }
        } else {
            quote! {
                let event_bus = container
                    .resolve::<::ironic::services::events::EventBus>()
                    .await
                    .map_err(|e| {
                        ::ironic::LifecycleError::new(
                            format!("EVENT_BUS_RESOLVE: {}", e),
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
