use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemStruct, Token, parse::Parse, parse::ParseStream, parse2};

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

    let secret_expr = attrs
        .secret_expr
        .ok_or_else(|| syn::Error::new(Span::call_site(), "`secret` is required for jwt_guard"))?;
    let claims_def = attrs
        .claims_struct
        .ok_or_else(|| syn::Error::new(Span::call_site(), "`claims` is required for jwt_guard"))?;
    let principal_def = attrs.principal_struct.ok_or_else(|| {
        syn::Error::new(Span::call_site(), "`principal` is required for jwt_guard")
    })?;
    let map_fn = attrs.map_fn.ok_or_else(|| {
        syn::Error::new(Span::call_site(), "`map` closure is required for jwt_guard")
    })?;

    let claims_ident = &claims_def.ident;
    let principal_ident = &principal_def.ident;

    // Make fields pub in the claims struct
    let claims_fields_pub = make_fields_pub(&claims_def.fields);
    let principal_fields = &principal_def.fields;

    let principal_id_field = principal_fields
        .named
        .first()
        .and_then(|f| f.ident.as_ref())
        .cloned()
        .unwrap_or_else(|| Ident::new("id", Span::call_site()));

    Ok(quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #[allow(missing_docs)]
        pub struct #claims_ident #claims_fields_pub

        #[allow(missing_docs)]
        pub struct #principal_ident #principal_fields

        impl ::ironic::auth::Principal for #principal_ident {
            fn subject(&self) -> &str {
                &self.#principal_id_field
            }
        }

        pub struct AuthGuard;

        impl ::ironic::Guard for AuthGuard {
            fn can_activate<'a>(
                &'a self,
                context: &'a mut ::ironic::RequestContext,
            ) -> ::ironic::GuardFuture<'a> {
                Box::pin(async move {
                    let secret = (#secret_expr).to_owned();
                    let token = ::ironic::auth::bearer_token(context.request())
                        .ok()
                        .flatten()
                        .unwrap_or_default();

                    if token.is_empty() {
                        return ::std::result::Result::Ok(::ironic::GuardDecision::Deny);
                    }

                    let jwt_service = ::ironic::auth::jwt::JwtService::hmac(
                        secret.as_bytes(),
                        ::ironic::auth::jwt::driver::Algorithm::HS256,
                    );
                    let decode_result = jwt_service.decode::<#claims_ident>(token);

                    match decode_result {
                        ::std::result::Result::Ok(data) => {
                            let map: &dyn ::std::ops::Fn(#claims_ident) -> ::std::result::Result<#principal_ident, ::ironic::auth::AuthError> = &#map_fn;
                            match map(data.claims) {
                                ::std::result::Result::Ok(_principal) => {
                                    ::std::result::Result::Ok(::ironic::GuardDecision::Allow)
                                }
                                ::std::result::Result::Err(_) => {
                                    ::std::result::Result::Ok(::ironic::GuardDecision::Deny)
                                }
                            }
                        }
                        ::std::result::Result::Err(_) => {
                            ::std::result::Result::Ok(::ironic::GuardDecision::Deny)
                        }
                    }
                })
            }
        }

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

fn make_fields_pub(fields: &syn::FieldsNamed) -> syn::FieldsNamed {
    let mut cloned = fields.clone();
    for field in &mut cloned.named {
        field.vis = syn::Visibility::Public(syn::token::Pub::default());
    }
    cloned
}
