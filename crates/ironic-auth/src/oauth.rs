//! OAuth 2.0 Authorization Code helpers with CSRF state and PKCE.

/// The upstream OAuth 2.0 API.
pub use ::oauth2 as driver;

use driver::{
    AsyncHttpClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
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

/// Exchanged token response from an OAuth provider.
#[derive(Clone, Debug)]
pub struct ProviderTokenResponse {
    /// The access token string.
    pub access_token: String,
    /// Optional refresh token.
    pub refresh_token: Option<String>,
    /// Token type (typically "Bearer").
    pub token_type: String,
    /// Expiry duration from the provider, if provided.
    pub expires_in: Option<std::time::Duration>,
    /// Scopes granted by the provider.
    pub scopes: Vec<String>,
}

/// Exchanges an authorization code for tokens using PKCE.
///
/// The caller provides an HTTP client that implements [`AsyncHttpClient`].
/// For `reqwest`, use `reqwest::Client::builder()
/// .redirect(reqwest::redirect::Policy::none()).build()`.
///
/// # Errors
///
/// Returns a string error description on network or provider failure.
pub async fn exchange_code<'c, C: AsyncHttpClient<'c>>(
    client: &'c ConfiguredBasicClient,
    code: &str,
    pkce_verifier: PkceCodeVerifier,
    http_client: &'c C,
) -> Result<ProviderTokenResponse, String> {
    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_owned()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(http_client)
        .await
        .map_err(|e| format!("Token exchange failed: {e}"))?;

    Ok(ProviderTokenResponse {
        access_token: token_response.access_token().secret().to_owned(),
        refresh_token: token_response
            .refresh_token()
            .map(|rt| rt.secret().to_owned()),
        token_type: token_response.token_type().as_ref().to_owned(),
        expires_in: token_response.expires_in(),
        scopes: token_response
            .scopes()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.as_ref().to_owned())
            .collect(),
    })
}

/// Validates that the callback `state` parameter matches the stored CSRF token.
///
/// Returns `Ok(())` on match, or an error message on mismatch.
///
/// # Errors
///
/// Returns an error when the state parameter does not match, indicating a
/// potential CSRF attack.
pub fn validate_state(callback_state: Option<&str>, stored_csrf: &CsrfToken) -> Result<(), String> {
    let state = callback_state
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Missing state parameter in callback".to_string())?;
    if state == stored_csrf.secret() {
        Ok(())
    } else {
        Err("State parameter mismatch — possible CSRF attack".to_string())
    }
}

/// Stores OAuth token information into a session for later use.
///
/// The tokens are persisted under the `oauth:access_token`,
/// `oauth:refresh_token`, `oauth:token_type`, and `oauth:scopes` keys.
///
/// # Errors
///
/// Returns a session error if JSON serialization fails.
#[cfg(feature = "sessions")]
pub fn store_tokens_in_session(
    session: &mut crate::auth::sessions::Session,
    tokens: &ProviderTokenResponse,
) -> Result<(), crate::auth::sessions::SessionError> {
    use crate::auth::sessions::SessionError;

    session.insert("oauth:access_token", &tokens.access_token)?;
    if let Some(refresh) = &tokens.refresh_token {
        session.insert("oauth:refresh_token", refresh)?;
    }
    session.insert("oauth:token_type", &tokens.token_type)?;
    if let Some(expires) = tokens.expires_in {
        let secs: u64 = expires.as_secs();
        session
            .insert("oauth:expires_in_secs", secs)
            .map_err(|_| SessionError::Store("serialization failed".into()))?;
    }
    session.insert("oauth:scopes", tokens.scopes.join(" "))?;
    Ok(())
}
