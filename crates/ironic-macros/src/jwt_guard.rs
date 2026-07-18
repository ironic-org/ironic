//! Generates the complete JWT auth pipeline from a concise declaration.
//!
//! # Example
//!
//! ```ignore
//! use ironic::jwt_guard;
//!
//! #[jwt_guard(
//!     secret = "JWT_SECRET",
//!     claims = UserClaims { sub: String, exp: u64 },
//!     principal = User { id: String },
//!     map = |c: UserClaims| -> Result<User, ironic::auth::AuthError> {
//!         Ok(User { id: c.sub })
//!     }
//! )]
//! pub struct Auth;
//! ```
//!
//! This generates:
//! - `UserClaims` struct with `Serialize + Deserialize`
//! - `User` struct with `Principal` impl
//! - `AuthGuard` type alias for `RequireAuthenticated<User>`
//! - `auth_middleware()` convenience fn returning `AuthenticationMiddleware`

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Ident, ItemStruct, Token, parse::Parse, parse::ParseStream, parse2,
};

#[derive(Default)]
struct JwtGuardArgs {
    secret_expr: Option<syn::Expr>,
    claims_struct: Option<ClaimsDef>,
    principal_struct: Option<PrincipalDef>,
    map_fn: Option<syn::ExprClosure>,
}

#[derive(Clone)]
struct ClaimsDef {
    ident: Ident,
    fields: syn::FieldsNamed,
}

#[derive(Clone)]
struct PrincipalDef {
    ident: Ident,
    fields: syn::FieldsNamed,
}

// Parse a struct-like body: `Name { field: Type, ... }`
fn parse_struct_body(input: syn::parse::ParseStream<'_>) -> syn::Result<(Ident, syn::FieldsNamed)> {
    let ident: Ident = input.parse()?;
    let fields: syn::FieldsNamed = if input.peek(syn::token::Brace) {
        input.parse()?
    } else {
        return Err(input.error("expected struct body `{ ... }`"));
    };
    Ok((ident, fields))
}

impl Parse for JwtGuardArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "secret" => {
                    args.secret_expr = Some(input.parse()?);
                }
                "claims" => {
                    let (ident, fields) = parse_struct_body(input)?;
                    args.claims_struct = Some(ClaimsDef { ident, fields });
                }
                "principal" => {
                    let (ident, fields) = parse_struct_body(input)?;
                    args.principal_struct = Some(PrincipalDef { ident, fields });
                }
                "map" => {
                    args.map_fn = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "expected `secret`, `claims`, `principal`, or `map`",
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

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let attrs = parse2::<JwtGuardArgs>(attr)?;
    let input = parse2::<ItemStruct>(item)?;
    let _config_name = &input.ident;

    let secret_expr = attrs.secret_expr.ok_or_else(|| {
        syn::Error::new(Span::call_site(), "`secret` is required for jwt_guard")
    })?;
    let claims_def = attrs.claims_struct.ok_or_else(|| {
        syn::Error::new(Span::call_site(), "`claims` is required for jwt_guard")
    })?;
    let principal_def = attrs.principal_struct.ok_or_else(|| {
        syn::Error::new(Span::call_site(), "`principal` is required for jwt_guard")
    })?;
    let map_fn = attrs.map_fn.ok_or_else(|| {
        syn::Error::new(Span::call_site(), "`map` closure is required for jwt_guard")
    })?;

    let claims_ident = &claims_def.ident;
    let claims_fields = &claims_def.fields;
    let principal_ident = &principal_def.ident;
    let principal_fields = &principal_def.fields;

    let principal_id_field = principal_fields
        .named
        .first()
        .and_then(|f| f.ident.as_ref())
        .cloned()
        .unwrap_or_else(|| Ident::new("id", Span::call_site()));

    Ok(quote! {
        #[derive(
            ::ironic::__private::serde_json::ser::Serialize,
            ::ironic::__private::serde_json::de::Deserialize,
        )]
        pub struct #claims_ident #claims_fields

        pub struct #principal_ident #principal_fields

        impl ::ironic::auth::Principal for #principal_ident {
            fn subject(&self) -> &str {
                &self.#principal_id_field
            }
        }

        /// JWT-based authentication guard for all routes.
        pub type AuthGuard = ::ironic::auth::RequireAuthenticated<#principal_ident>;

        /// Convenience function: creates the authentication middleware.
        ///
        /// Reads the secret from the expression provided in `#[jwt_guard(secret = ...)]`.
        pub fn auth_middleware() -> ::ironic::auth::AuthenticationMiddleware<
            ::ironic::auth::jwt::JwtBearerAuthenticator<
                #claims_ident,
                #principal_ident,
                impl ::std::ops::Fn(#claims_ident) -> ::std::result::Result<#principal_ident, ::ironic::auth::AuthError> + Send + Sync + 'static,
            >,
            #principal_ident,
        > {
            let secret = (#secret_expr).to_owned();
            let jwt_service = ::ironic::auth::jwt::JwtService::hmac(
                secret.as_bytes(),
                ::ironic::auth::jwt::driver::Algorithm::HS256,
            );
            let authenticator = ::ironic::auth::jwt::JwtBearerAuthenticator::new(
                jwt_service,
                #map_fn,
            );
            ::ironic::auth::AuthenticationMiddleware::new(authenticator)
        }

        #input
    })
}
