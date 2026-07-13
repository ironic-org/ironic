//! OAuth 2.0 Authorization Code helpers with CSRF state and PKCE.

/// The upstream OAuth 2.0 API.
pub use ::oauth2 as driver;

use driver::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl, basic::BasicClient,
};

/// A basic client with authorization and token endpoints configured.
pub type ConfiguredBasicClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

/// Creates a configured basic OAuth 2.0 client.
///
/// # Errors
///
/// Returns a URL parse error when any endpoint is invalid.
pub fn basic_client(
    client_id: impl Into<String>,
    client_secret: Option<String>,
    authorization_url: impl Into<String>,
    token_url: impl Into<String>,
    redirect_url: impl Into<String>,
) -> Result<ConfiguredBasicClient, driver::url::ParseError> {
    let mut client = BasicClient::new(ClientId::new(client_id.into()));
    if let Some(secret) = client_secret {
        client = client.set_client_secret(ClientSecret::new(secret));
    }
    Ok(client
        .set_auth_uri(AuthUrl::new(authorization_url.into())?)
        .set_token_uri(TokenUrl::new(token_url.into())?)
        .set_redirect_uri(RedirectUrl::new(redirect_url.into())?))
}

/// Values required to begin and later validate an authorization-code flow.
#[derive(Debug)]
pub struct AuthorizationRequest {
    /// URL to which the user agent should be redirected.
    pub url: driver::url::Url,
    /// Secret state that must match the callback's `state` parameter.
    pub csrf_state: CsrfToken,
    /// PKCE verifier that must be supplied when exchanging the callback code.
    pub pkce_verifier: PkceCodeVerifier,
}

/// Builds an authorization URL with fresh CSRF state and an S256 PKCE challenge.
#[must_use]
pub fn authorization_request(
    client: &ConfiguredBasicClient,
    scopes: impl IntoIterator<Item = String>,
) -> AuthorizationRequest {
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(challenge);
    for scope in scopes {
        request = request.add_scope(Scope::new(scope));
    }
    let (url, csrf_state) = request.url();
    AuthorizationRequest {
        url,
        csrf_state,
        pkce_verifier: verifier,
    }
}
