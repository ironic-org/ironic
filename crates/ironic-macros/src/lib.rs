//! Procedural macros for declaring Ironic application metadata.

use proc_macro::TokenStream;

mod controller;
mod from_row;
mod injectable;
mod jwt_guard;
mod merge;
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
    cron,
    interval,
    timeout,
    api,
    resp,
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
