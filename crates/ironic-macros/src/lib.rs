#![allow(
    clippy::type_complexity,
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::match_wildcard_for_single_variants
)]
//! Procedural macros for declaring Ironic application metadata.

use proc_macro::TokenStream;

mod controller;
mod event_handler;
mod from_row;
mod graphql;
mod injectable;
mod jwt_guard;
mod mapped_types;
mod mcp_tool;
mod merge;
mod message_handler;
mod module;
mod openapi;
mod routes;
mod serializable;
mod r#test;
mod ws_gateway;

#[proc_macro_derive(Injectable, attributes(injectable))]
/// Derives a dependency-injection provider definition.
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    injectable::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Module, attributes(module, global))]
/// Derives a static application module definition.
pub fn derive_module(input: TokenStream) -> TokenStream {
    module::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Marks a struct as a GraphQL resolver with DI injection.
///
/// The struct is automatically registered as an `#[Injectable]` and can be
/// used in GraphQL schema building.
pub fn resolver(attribute: TokenStream, item: TokenStream) -> TokenStream {
    graphql::expand_resolver(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Marks a method as a GraphQL query field.
pub fn gql_query(attribute: TokenStream, item: TokenStream) -> TokenStream {
    graphql::expand_query(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Marks a method as a GraphQL mutation field.
pub fn mutation(attribute: TokenStream, item: TokenStream) -> TokenStream {
    graphql::expand_mutation(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Marks a method as a GraphQL subscription field.
pub fn subscription(attribute: TokenStream, item: TokenStream) -> TokenStream {
    graphql::expand_subscription(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(FromRow, attributes(sqlx))]
/// Derives `sqlx::FromRow` for a named-field struct with optional column rename,
/// JSON-deserialization, and default-value support.
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    from_row::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Merge)]
/// Derives a `merge_into(&mut self)` method that applies `Option<T>` values
/// from `self` onto a target of the same type.
pub fn derive_merge(input: TokenStream) -> TokenStream {
    merge::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(PartialType, attributes(partial))]
/// Derives an OpenAPI schema with all fields made optional from a base type.
///
/// # Example
///
/// ```ignore
/// #[derive(PartialType)]
/// #[partial(CreateUserDto)]
/// struct UpdateUserDto;
/// ```
pub fn derive_partial_type(input: TokenStream) -> TokenStream {
    mapped_types::expand_partial(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(PickType, attributes(pick))]
/// Derives an OpenAPI schema that includes only the specified fields from a base type.
///
/// # Example
///
/// ```ignore
/// #[derive(PickType)]
/// #[pick(User, fields = ["id", "name", "email"])]
/// struct UserResponse;
/// ```
pub fn derive_pick_type(input: TokenStream) -> TokenStream {
    mapped_types::expand_pick(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(OmitType, attributes(omit))]
/// Derives an OpenAPI schema that excludes the specified fields from a base type.
///
/// # Example
///
/// ```ignore
/// #[derive(OmitType)]
/// #[omit(User, fields = ["password_hash"])]
/// struct SafeUser;
/// ```
pub fn derive_omit_type(input: TokenStream) -> TokenStream {
    mapped_types::expand_omit(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(OpenApiSchema, attributes(serde, garde))]
/// Derives an `OpenAPI` schema for a named-field struct.
///
/// Reads `#[serde(rename)]`, `#[serde(skip)]`, `#[serde(default)]`, and
/// `#[garde(...)]` attributes to produce richer schema metadata.
pub fn derive_openapi_schema(input: TokenStream) -> TokenStream {
    openapi::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Generates the complete JWT auth pipeline (claims, principal, authenticator,
/// guard, and middleware) from a concise declaration.
///
/// # Example
///
/// ```ignore
/// #[ironic::jwt_guard(
///     secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
///     claims = UserClaims { sub: String, exp: u64 },
///     principal = User { id: String },
///     map = |c: UserClaims| -> Result<User, ironic::auth::AuthError> {
///         Ok(User { id: c.sub })
///     }
/// )]
/// pub struct Auth;
///
/// // Use in application setup:
/// app.middleware(Auth::auth_middleware());
/// // And on controllers:
/// #[guard(Auth::AuthGuard)]
/// ```
pub fn jwt_guard(attribute: TokenStream, item: TokenStream) -> TokenStream {
    jwt_guard::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Generates an [`McpTool`] from an async function.
///
/// The function becomes an MCP tool that AI agents can discover and call.
/// Parameters are automatically converted to a JSON Schema for the input.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use ironic::{mcp_tool, McpRouter, AxumAdapter};
///
/// #[mcp_tool("greet", description = "Greets a user by name")]
/// async fn greet(name: String) -> Result<String, String> {
///     Ok(format!("Hello, {name}!"))
/// }
///
/// let adapter = AxumAdapter::new()
///     .mcp(McpRouter::new().register_tool(mcp_tool_greet()));
/// ```
pub fn mcp_tool(attribute: TokenStream, item: TokenStream) -> TokenStream {
    mcp_tool::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Registers an async function as a message handler on a `MicroserviceServer`.
///
/// The handler receives a deserialized request and returns a response that is
/// serialized and sent back to the caller. Use with a microservice server that
/// implements the request-response pattern.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use ironic::distributed::{MicroserviceServer, MessageContext};
///
/// #[message_handler("user.get")]
/// async fn get_user(request: GetUserRequest) -> GetUserResponse {
///     // ...
/// }
/// ```
pub fn message_handler(attribute: TokenStream, item: TokenStream) -> TokenStream {
    message_handler::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Registers an async function as an event handler on the application's `EventBus`.
///
/// The event type is inferred from the method's single parameter (supports `Arc<E>`).
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use ironic::services::events::{EventBus, EventSubscription};
///
/// #[event_handler(capacity = 64)]
/// async fn handle_order_placed(event: Arc<String>) {
///     tracing::info!("received event: {event}");
/// }
///
/// let bus = EventBus::default();
/// __event_handler_reg_handle_order_placed(&bus);
/// ```
pub fn event_handler(attribute: TokenStream, item: TokenStream) -> TokenStream {
    event_handler::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Declares a controller and its path prefix.
pub fn controller(attribute: TokenStream, item: TokenStream) -> TokenStream {
    controller::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
/// Collects route metadata from an inherent controller implementation.
pub fn routes(attribute: TokenStream, item: TokenStream) -> TokenStream {
    routes::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Serializable, attributes(exclude, expose))]
/// Derives a `field_rules()` method from `#[exclude]` and `#[expose(role = "...")]`
/// field attributes.
pub fn derive_serializable(input: TokenStream) -> TokenStream {
    serializable::expand(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

macro_rules! marker_attribute {
    ($($name:ident),+ $(,)?) => {$ (
        #[doc = concat!("Route metadata consumed by [`macro@routes`].")]
        #[proc_macro_attribute]
        pub fn $name(_attribute: TokenStream, item: TokenStream) -> TokenStream {
            item
        }
    )+};
}

#[proc_macro_attribute]
/// Declares a WebSocket gateway and its path.
pub fn web_socket_gateway(attribute: TokenStream, item: TokenStream) -> TokenStream {
    ws_gateway::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

marker_attribute!(
    get,
    post,
    put,
    patch,
    delete,
    head,
    options,
    body,
    form,
    query,
    param,
    header,
    decorator,
    pipe,
    subscribe_message,
    guard,
    interceptor,
    middleware,
    cache,
    cache_key,
    cache_ttl,
    cron,
    interval,
    timeout,
    api,
    resp,
    sse,
    forward_ref,
    raw_body,
    cookie,
);

/// Wraps an async test function with Ironic's Tokio runtime, removing the
/// need for users to depend on `tokio` or use `#[tokio::test]`.
///
/// # Usage
///
/// ```ignore
/// use ironic::test;
///
/// #[test]
/// async fn my_test() {
///     // test body — no tokio dependency needed
/// }
/// ```
#[proc_macro_attribute]
pub fn r#test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    r#test::expand(attribute.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Configures an async entry point with Ironic's Tokio runtime.
#[proc_macro_attribute]
pub fn main(attribute: TokenStream, item: TokenStream) -> TokenStream {
    if !attribute.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`#[ironic::main]` does not accept arguments",
        )
        .into_compile_error()
        .into();
    }
    let mut function = syn::parse_macro_input!(item as syn::ItemFn);
    if function.sig.asyncness.is_none() {
        let error = syn::Error::new_spanned(
            function.sig.fn_token,
            "`#[ironic::main]` requires an async function",
        )
        .into_compile_error();
        return quote::quote!(#error #function).into();
    }
    if !function.sig.inputs.is_empty() {
        let error = syn::Error::new_spanned(
            &function.sig.inputs,
            "`#[ironic::main]` entry points cannot accept arguments",
        )
        .into_compile_error();
        return quote::quote!(#error #function).into();
    }
    function.sig.asyncness = None;
    let body = function.block;
    function.block = Box::new(syn::parse_quote!({
        ::ironic::__private::block_on(async move #body)
    }));
    quote::quote!(#function).into()
}
