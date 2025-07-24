/// Oauth authorization for MCP. The sequence diagram at
/// https://modelcontextprotocol.io/specification/draft/basic/authorization#authorization-flow-steps
/// is a useful resource.
use crate::{
    api::{
        self, ApiResult, ErrorResponse, IntoApiResult, bad_request_error,
        google::{self, User},
    },
    settings::settings,
};
use anyhow::{Context as _, Result, anyhow};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{self, Form},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{
    Engine as _,
    prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tower_http::cors::{self, CorsLayer};
use uuid::Uuid;

/// Implements MCP oauth: https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization
pub(crate) fn router() -> Result<Router> {
    let store = Store {
        clients: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors_layer = CorsLayer::new()
        .allow_origin(cors::Any)
        .allow_methods(cors::Any)
        .allow_headers(cors::Any);

    Ok(Router::new()
        .nest(
            "/.well-known/oauth-authorization-server",
            Router::new()
                .route(
                    "/api/mcp/sse",
                    get(get_authorization_server_metadata)
                        .options(get_authorization_server_metadata),
                )
                .route(
                    "/",
                    get(get_authorization_server_metadata)
                        .options(get_authorization_server_metadata),
                )
                .fallback(api::handler_404)
                .layer(cors_layer.clone()),
        )
        .nest(
            "/.well-known/oauth-protected-resource",
            Router::new()
                .route(
                    "/api/mcp/sse",
                    get(get_resource_server_metadata).options(get_resource_server_metadata),
                )
                .route(
                    "/",
                    get(get_resource_server_metadata).options(get_resource_server_metadata),
                )
                .fallback(api::handler_404)
                .layer(cors_layer.clone()),
        )
        .nest(
            "/oauth",
            Router::new()
                .route("/register", post(oauth_register).options(oauth_register))
                .route("/token", post(oauth_token).options(oauth_token))
                .layer(cors_layer)
                .route("/authorization_details", post(oauth_authorization_details))
                .route(
                    "/approve",
                    post(oauth_approve)
                        .options(oauth_approve)
                        .layer(middleware::from_fn(google::authenticate)),
                )
                .layer((Extension(store), middleware::from_fn(set_cache_control)))
                .fallback(api::handler_404),
        ))
}

/// Middleware function that authenticates requests to the MCP server
/// by looking for a Bearer token in the Authorization header.
#[tracing::instrument(skip(decoding_key, req, next), fields(email))]
pub(crate) async fn authenticate(
    Extension(decoding_key): Extension<DecodingKey>,
    mut req: extract::Request,
    next: Next,
) -> OauthResult<Response<Body>> {
    let access_token_claims: AccessTokenClaims = _authenticate(&decoding_key, &mut req)
        .context_status(
            StatusCode::UNAUTHORIZED,
            // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#name-error-codes
            "invalid_token",
            "Unauthenticated client",
        )?;

    let mut user = access_token_claims.user;
    user.email = user.email.to_lowercase();

    tracing::Span::current().record("email", user.email.clone());
    assert!(req.extensions_mut().insert(user).is_none());

    Ok(next.run(req).await)
}

fn _authenticate(
    decoding_key: &DecodingKey,
    request: &mut extract::Request,
) -> Result<AccessTokenClaims> {
    // Parse the access token out of the Authorization header.
    let access_token = {
        let headers = request.headers();
        let Some(auth_header) = headers.get("Authorization") else {
            return Err(anyhow!("Authorization header is absent"));
        };

        let Ok(auth) = auth_header.to_str() else {
            return Err(anyhow!(
                "Could not convert auth header to string: {auth_header:?}"
            ));
        };
        let parts: Vec<&str> = auth.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Err(anyhow!("Could not split bearer parts: {parts:?}"));
        }
        parts[1]
    };

    // Decode the access token.
    decode_access_token(decoding_key, access_token)
}

const READ_WRITE_SCOPE: &str = "read_write";
const BASIC_AUTH_METHOD: &str = "client_secret_basic";
const POST_AUTH_METHOD: &str = "client_secret_post";
const NONE_AUTH_METHOD: &str = "none";
const AUTH_METHODS_SUPPORTED: &[&str] = &[BASIC_AUTH_METHOD, POST_AUTH_METHOD, NONE_AUTH_METHOD];
const CODE_RESPONSE_TYPE: &str = "code";
const CODE_GRANT_TYPE: &str = "authorization_code";
const REFRESH_GRANT_TYPE: &str = "refresh_token";
const S256_CHALLENGE_METHOD: &str = "S256";

/// oauth2 resource server metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ResourceServerMetadata {
    /// https://www.rfc-editor.org/rfc/rfc8707.html
    resource: String,
    authorization_servers: Vec<String>,
    bearer_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

/// https://datatracker.ietf.org/doc/rfc9728/
#[tracing::instrument()]
async fn get_resource_server_metadata() -> OauthResult<Json<ResourceServerMetadata>> {
    let host = &settings().host;
    let metadata = ResourceServerMetadata {
        resource: format!("{host}/api/mcp/sse"),
        authorization_servers: vec![host.clone()],
        bearer_methods_supported: vec!["header".to_string()],
        scopes_supported: vec![READ_WRITE_SCOPE.to_string()],
    };
    tracing::trace!("Resource server metadata: {metadata:?}");

    Ok(Json(metadata))
}

/// oauth2 authorization server metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuthorizationServerMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
    token_endpoint_auth_methods_supported: Vec<String>,
    registration_endpoint: String,
    issuer: String,
    scopes_supported: Vec<String>,
    grant_types_supported: Vec<String>,
    response_types_supported: Vec<String>,
    code_challenge_methods_supported: Vec<String>,
}

/// https://datatracker.ietf.org/doc/html/rfc8414
#[tracing::instrument()]
async fn get_authorization_server_metadata() -> OauthResult<Json<AuthorizationServerMetadata>> {
    let host = &settings().host;
    let metadata = AuthorizationServerMetadata {
        authorization_endpoint: format!("{host}/connections/mcp/oauth/authorize"),
        token_endpoint: format!("{host}/oauth/token"),
        token_endpoint_auth_methods_supported: AUTH_METHODS_SUPPORTED
            .iter()
            .map(|s| s.to_string())
            .collect(),
        registration_endpoint: format!("{host}/oauth/register"),
        issuer: host.clone(),
        scopes_supported: vec![READ_WRITE_SCOPE.to_string()],
        grant_types_supported: vec![CODE_GRANT_TYPE.to_string()],
        response_types_supported: vec![CODE_RESPONSE_TYPE.to_string()],
        code_challenge_methods_supported: vec![S256_CHALLENGE_METHOD.to_string()],
    };
    tracing::trace!("Authorization server metadata: {metadata:?}");

    Ok(Json(metadata))
}

#[derive(Clone)]
struct Store {
    clients: Arc<Mutex<HashMap<String, ClientSecretClaims>>>,
}

// TODO: Consider storing clients in the DB such that restarts don't evict.
impl Store {
    async fn insert_client(&self, claims: &ClientSecretClaims) -> ApiResult<()> {
        let mut clients = self.clients.lock().await;
        // Drop all clients to avoid an OOM. The limit is sufficiently high such
        // we're unlikely to hit it between server restarts.
        if clients.len() > 7500 {
            tracing::error!("Cleared oauth clients. Consider implementing better eviction.");
            clients.clear();
            clients.shrink_to_fit();
        }
        match clients.entry(claims.client_id.clone()) {
            Entry::Occupied(entry) => {
                return Err(anyhow!("Client already exists: {}", entry.key()).into());
            }
            Entry::Vacant(entry) => {
                entry.insert(claims.clone());
            }
        }

        Ok(())
    }

    async fn get_client(&self, client_id: &str) -> Option<ClientSecretClaims> {
        self.clients.lock().await.get(client_id).cloned()
    }
}

/// Note: All fields must be optional. Validation should occur with the handler.
/// See `swap_empty_with_none` below.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientRegistrationRequest {
    client_name: Option<String>,
    scope: Option<String>,
    redirect_uris: Vec<String>,
    token_endpoint_auth_method: Option<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
struct ClientRegistrationResponse {
    client_id: String,
    client_name: String,
    scope: String,
    token_endpoint_auth_method: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    client_secret_expires_at: u64,
    client_secret: String,
}

// Handle dynamic client registration
// https://datatracker.ietf.org/doc/html/rfc7591#section-3.1
#[tracing::instrument(skip(key, store, req))]
async fn oauth_register(
    Extension(key): Extension<EncodingKey>,
    Extension(store): Extension<Store>,
    Json(req): Json<ClientRegistrationRequest>,
) -> OauthResult<Json<ClientRegistrationResponse>> {
    let req = trim_client_registration_request(req);
    tracing::debug!("Registering client: {req:?}");

    // Validate the request
    let redirect_uris = req.redirect_uris;
    if redirect_uris.is_empty() {
        return Err(bad_request_error(
            "invalid_redirect_uri",
            "At least one redirect uri is required",
        )
        .into());
    }
    if redirect_uris.len() > 15 {
        return Err(bad_request_error("invalid_redirect_uri", "Too many redirect uris").into());
    }
    if redirect_uris.iter().any(|s| s.is_empty()) {
        return Err(
            bad_request_error("invalid_redirect_uri", "Redirect uri cannot be empty").into(),
        );
    }
    if redirect_uris.iter().any(|s| s.len() > 2000) {
        return Err(bad_request_error("invalid_redirect_uri", "Redirect uri too long").into());
    }
    let response_types = req.response_types;
    if !response_types.is_empty() && !response_types.contains(&CODE_RESPONSE_TYPE.to_string()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only the 'code' response_type is supported",
        )
        .into());
    }
    let grant_types = req.grant_types;
    if !grant_types.is_empty() && !grant_types.contains(&CODE_GRANT_TYPE.to_string()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "grant_types must contain 'authorization_code'",
        )
        .into());
    }
    let scope = req.scope.unwrap_or_else(|| READ_WRITE_SCOPE.to_string());
    if scope != READ_WRITE_SCOPE {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only read_write scope is supported",
        )
        .into());
    }
    let token_endpoint_auth_method = req
        .token_endpoint_auth_method
        .unwrap_or_else(|| BASIC_AUTH_METHOD.to_string());
    if !AUTH_METHODS_SUPPORTED.contains(&token_endpoint_auth_method.as_str()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only client_secret_basic, client_secret_post and none token_auth_method are supported",
        )
        .into());
    }

    let client_id = format!("client-{}", Uuid::new_v4());
    let mut client_name = req.client_name.unwrap_or_else(|| client_id.clone());
    if client_name.len() > 255 {
        client_name.truncate(255);
        client_name = format!("{client_name}..");
    }

    // Generate the client secret.
    let claims = ClientSecretClaims {
        exp: expires_at(CLIENT_SECRET_EXPIRY_SECS)?,
        expires_in: CLIENT_SECRET_EXPIRY_SECS,
        iss: CLIENT_SECRET_ISS.to_string(),
        client_id,
        client_name,
        scope,
        token_endpoint_auth_method,
        redirect_uris,
        grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
        response_types: vec![CODE_RESPONSE_TYPE.to_string()],
    };
    let client_secret = encode_client_secret(&key, &claims)?;
    store.insert_client(&claims).await?;

    // Create the response.
    let response = ClientRegistrationResponse {
        client_id: claims.client_id,
        client_name: claims.client_name,
        scope: claims.scope,
        token_endpoint_auth_method: claims.token_endpoint_auth_method,
        redirect_uris: claims.redirect_uris,
        grant_types: claims.grant_types,
        response_types: claims.response_types,
        client_secret_expires_at: claims.exp,
        client_secret,
    };
    tracing::debug!("Registered client: {response:?}");

    Ok(Json(response))
}

/// Note: All fields must be optional. Validation should occur with the handler.
/// See `swap_empty_with_none` below.
#[derive(Debug, Deserialize)]
struct AuthorizationDetailsRequest {
    client_id: Option<String>,
    redirect_uri: Option<String>,
    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AuthorizationDetailsResponse {
    client_name: String,
}

#[tracing::instrument(skip(store, req))]
async fn oauth_authorization_details(
    Extension(store): Extension<Store>,
    Json(req): Json<AuthorizationDetailsRequest>,
) -> ApiResult<Json<AuthorizationDetailsResponse>> {
    let req = trim_authorize_request(req);
    tracing::info!("Handling authorization: {req:?}");

    // Validate the request.
    let Some(client_id) = req.client_id else {
        return Err(bad_request_error("invalid_request", "Client id required"));
    };
    let Some(redirect_uri) = req.redirect_uri else {
        return Err(bad_request_error(
            "invalid_request",
            "Redirect uri required",
        ));
    };

    // Validate that the client exists and has registered the given redirect uri.
    let Some(claims) = store.get_client(&client_id).await else {
        return Err(bad_request_error(
            "unauthorized_client",
            "Client is unregistered. Delete any dynamic clients and try again.",
        ));
    };
    if !claims.redirect_uris.contains(&redirect_uri) {
        return Err(bad_request_error(
            "invalid_request",
            "Registered redirect uri doesn't match the provided. Danger!",
        ));
    }

    Ok(Json(AuthorizationDetailsResponse {
        client_name: claims.client_name,
    }))
}

/// Note: All fields must be optional. Validation should occur with the handler.
/// See `swap_empty_with_none` below.
#[derive(Debug, Deserialize)]
struct ApprovalRequest {
    client_id: Option<String>,
    scope: Option<String>,
    response_type: Option<String>,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    redirect_uri: Option<String>,
    /// https://www.rfc-editor.org/rfc/rfc8707.html#name-resource-parameter
    #[allow(dead_code)]
    resource: Option<String>,
    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ApprovalResponse {
    code: String,
}

/// Handle approval requests sent from a client browser.
/// https://datatracker.ietf.org/doc/html/rfc6749#section-3.1
#[tracing::instrument(skip(user, store, encoding_key, req))]
async fn oauth_approve(
    Extension(user): Extension<User>,
    Extension(store): Extension<Store>,
    Extension(encoding_key): Extension<EncodingKey>,
    Json(req): Json<ApprovalRequest>,
) -> ApiResult<Json<ApprovalResponse>> {
    let req: ApprovalRequest = trim_approval_request(req);
    tracing::info!("Approving authorization: {req:?}");

    // Validate the request.
    let Some(client_id) = req.client_id else {
        return Err(bad_request_error("invalid_request", "Client id required"));
    };
    let Some(redirect_uri) = req.redirect_uri else {
        return Err(bad_request_error(
            "invalid_request",
            "Redirect uri required",
        ));
    };
    let scope = req.scope.unwrap_or_else(|| READ_WRITE_SCOPE.to_string());
    if scope != READ_WRITE_SCOPE {
        return Err(bad_request_error(
            "invalid_scope",
            "Only read_write scope is supported",
        ));
    }
    let response_type = req
        .response_type
        .unwrap_or_else(|| CODE_RESPONSE_TYPE.to_string());
    if response_type != CODE_RESPONSE_TYPE {
        return Err(bad_request_error(
            "unsupported_response_type",
            "Only the 'code' response_type is supported",
        ));
    }
    let (code_challenge_method, code_challenge) = (req.code_challenge_method, req.code_challenge);
    match (&code_challenge_method, &code_challenge) {
        (Some(method), Some(_challenge)) => {
            if method != S256_CHALLENGE_METHOD {
                return Err(bad_request_error(
                    "invalid_request",
                    "Only 'S256' code_challenge_method is supported",
                ));
            }
        }
        (None, None) => {}
        _ => {
            return Err(bad_request_error(
                "invalid_request",
                "Method and code must both be set or unset",
            ));
        }
    }
    // Validate that the client exists and has registered the given redirect uri.
    let Some(claims) = store.get_client(&client_id).await else {
        return Err(bad_request_error(
            "unauthorized_client",
            "Client is unregistered. Delete any dynamic clients and try again.",
        ));
    };
    if !claims.redirect_uris.contains(&redirect_uri) {
        return Err(bad_request_error(
            "invalid_request",
            "Registered redirect uri doesn't match the provided. Danger!",
        ));
    }

    // Encode the auth token
    let auth_token_claims = AuthTokenClaims {
        exp: expires_at(AUTH_TOKEN_EXPIRY_SECS)?,
        iss: AUTH_TOKEN_ISS.to_string(),
        client_id,
        session: format!("session-{}", Uuid::new_v4()),
        expires_in: AUTH_TOKEN_EXPIRY_SECS,
        scope,
        response_type,
        redirect_uri,
        code_challenge,
        code_challenge_method,
        user,
    };
    let auth_token = encode_auth_token(&encoding_key, &auth_token_claims)?;

    tracing::info!("Approved authorization, created auth token: {auth_token_claims:?}");

    Ok(Json(ApprovalResponse { code: auth_token }))
}

/// Note: All fields must be optional. Validation should occur with the handler.
/// See `swap_empty_with_none` below.
#[derive(Deserialize)]
struct TokenRequest {
    /// refresh_token or authorization_code
    grant_type: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    /// https://www.rfc-editor.org/rfc/rfc8707.html#name-resource-parameter
    #[allow(dead_code)]
    resource: Option<String>,
    // TODO: validate.
    #[allow(dead_code)]
    redirect_uri: Option<String>,

    // authorization_code fields
    code: Option<String>,
    code_verifier: Option<String>,

    // refresh_token_fields
    refresh_token: Option<String>,

    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Serialize)]
struct TokenResponse {
    token_type: String,
    expires_in: u64,
    scope: String,
    access_token: String,
    refresh_token: String,
}

/// Handle token request from the MCP client
/// https://datatracker.ietf.org/doc/html/rfc6749#section-3.2
#[tracing::instrument(skip(store, decoding_key, encoding_key, req, headers))]
async fn oauth_token(
    Extension(store): Extension<Store>,
    Extension(decoding_key): Extension<DecodingKey>,
    Extension(encoding_key): Extension<EncodingKey>,
    headers: HeaderMap,
    Form(req): Form<TokenRequest>,
) -> OauthResult<Json<TokenResponse>> {
    let req = trim_token_request(req);
    tracing::info!("Handling token request: {req:?}");

    // Authenticate the client or allow unauthenticated clients.
    let authenticated_client_id =
        authenticate_token_client(&headers, &req, &store, &decoding_key).await?;

    // Validate the refresh token or authorization code.
    let (client_id, scope, user, auth_token_claims) = match req.grant_type.as_deref() {
        Some(CODE_GRANT_TYPE) => validate_authorization_code(&req, &decoding_key),
        Some(REFRESH_GRANT_TYPE) => validate_refresh_token(&req, &decoding_key),
        Some(_) => Err(bad_request_error(
            "unsupported_grant_type",
            "only authorization_code is supported",
        )
        .into()),
        None => Err(bad_request_error("invalid_request", "grant_type required").into()),
    }?;
    if authenticated_client_id != client_id {
        return Err(bad_request_error("invalid_grant", "Grant issued for another client").into());
    }

    // Encode the new access token and refresh token.
    let access_claims = AccessTokenClaims {
        exp: expires_at(ACCESS_TOKEN_EXPIRY_SECS)?,
        iss: ACCESS_TOKEN_ISS.to_string(),
        client_id,
        expires_in: ACCESS_TOKEN_EXPIRY_SECS,
        scope,
        user,
        auth_token_claims,
    };
    let access_token = encode_access_token(&encoding_key, &access_claims)?;
    let refresh_claims = RefreshTokenClaims {
        exp: expires_at(REFRESH_TOKEN_EXPIRY_SECS)?,
        iss: REFRESH_TOKEN_ISS.to_string(),
        client_id: access_claims.client_id.clone(),
        expires_in: REFRESH_TOKEN_EXPIRY_SECS,
        scope: access_claims.scope.clone(),
        user: access_claims.user.clone(),
        auth_token_claims: access_claims.auth_token_claims.clone(),
    };
    let refresh_token = encode_refresh_token(&encoding_key, &refresh_claims)?;

    tracing::info!("Created access token: {access_claims:?}, refresh token: {refresh_claims:?}");

    Ok(Json(TokenResponse {
        token_type: "Bearer".to_string(),
        expires_in: access_claims.expires_in,
        scope: access_claims.scope,
        access_token,
        refresh_token,
    }))
}

fn validate_refresh_token(
    req: &TokenRequest,
    decoding_key: &DecodingKey,
) -> OauthResult<(String, String, User, AuthTokenClaims)> {
    // Decode the refresh token and grab the auth token.
    let Some(refresh_token) = &req.refresh_token else {
        return Err(bad_request_error(
            "invalid_request",
            "refresh_token required for refresh_token grant type",
        )
        .into());
    };
    let refresh_token = decode_refresh_token(decoding_key, refresh_token)
        .context_bad_request("invalid_grant", "Invalid refresh token")?;

    // Validate the token was issued for this client
    // The check is optional as client_id may be omitted for refresh_token requests.
    if let Some(client_id) = &req.client_id
        && client_id != &refresh_token.client_id
    {
        return Err(
            bad_request_error("invalid_grant", "Refresh token issued to another client").into(),
        );
    }

    Ok((
        refresh_token.client_id,
        refresh_token.scope,
        refresh_token.user,
        refresh_token.auth_token_claims,
    ))
}

fn validate_authorization_code(
    req: &TokenRequest,
    decoding_key: &DecodingKey,
) -> OauthResult<(String, String, User, AuthTokenClaims)> {
    // Decode the auth token.
    let Some(code) = &req.code else {
        return Err(bad_request_error(
            "invalid_request",
            "code required for authorization_code grant type",
        )
        .into());
    };
    let auth_token_claims = decode_auth_token(decoding_key, code)
        .context_bad_request("invalid_grant", "Invalid auth token")?;

    // PKCE: Verify the challenge against the verifier.
    validate_code_challenge(req, &auth_token_claims)?;

    // Validate the token was issued for this client
    // The check is optional as client_id may be omitted for authenticated auth_token requests.
    if let Some(client_id) = &req.client_id
        && client_id != &auth_token_claims.client_id
    {
        return Err(
            bad_request_error("invalid_grant", "Auth code issued to another client").into(),
        );
    }

    Ok((
        auth_token_claims.client_id.clone(),
        auth_token_claims.scope.clone(),
        auth_token_claims.user.clone(),
        auth_token_claims,
    ))
}

/// Implement PKCE.
/// See https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#section-4.1.1
fn validate_code_challenge(
    req: &TokenRequest,
    auth_token_claims: &AuthTokenClaims,
) -> OauthResult<()> {
    match (
        &auth_token_claims.code_challenge_method,
        &auth_token_claims.code_challenge,
        &req.code_verifier,
    ) {
        (Some(method), Some(challenge), Some(verifier)) => {
            if method != S256_CHALLENGE_METHOD {
                return Err(
                    bad_request_error("unsupported_grant_type", "Only S256 is supported").into(),
                );
            }
            let actual_challenge =
                BASE64_URL_SAFE_NO_PAD.encode(Sha256::new().chain_update(verifier).finalize());
            if &actual_challenge != challenge {
                return Err(bad_request_error(
                    "invalid_grant",
                    "Challenge does not match verifier",
                )
                .into());
            }
        }
        (None, None, None) => {}
        _ => {
            return Err(bad_request_error(
                "invalid_grant",
                "Method, challenge and verifier must all be set or unset",
            )
            .into());
        }
    }

    Ok(())
}

async fn authenticate_token_client(
    headers: &HeaderMap,
    req: &TokenRequest,
    store: &Store,
    decoding_key: &DecodingKey,
) -> OauthResult<String> {
    match (headers.get("Authorization"), &req.client_secret) {
        // client_secret_basic
        // Prefer to use the Authorization header.
        (Some(header), _) => authenticate_token_client_basic(req, header, decoding_key),

        // client_secret_post
        // Use the request body when the Authorization header is absent.
        (None, Some(client_secret)) => {
            authenticate_token_client_post(req, client_secret, decoding_key)
        }

        // none
        // Unauthenticated clients.
        (None, None) => validate_unauthenticated_client(req, store).await,
    }
}

/// Authenticate the client using client_secret_basic auth.
/// i.e. Authorization header Basic
fn authenticate_token_client_basic(
    req: &TokenRequest,
    header: &HeaderValue,
    decoding_key: &DecodingKey,
) -> OauthResult<String> {
    tracing::debug!("Authenticating with client_secret_basic Authorization header");

    let Some(credentials) = header.as_bytes().strip_prefix(b"Basic ") else {
        return Err(bad_request_error("invalid_client", "Invalid authorization header").into());
    };
    let credentials: String = BASE64_STANDARD
        .decode(credentials)
        .context_bad_request("invalid_client", "Invalid authorization credentials")?
        .try_into()
        .context_bad_request("invalid_client", "Invalid authorization credentials")?;
    let mut credentials = credentials.split(":");
    let Some(client_id) = credentials.next() else {
        return Err(
            bad_request_error("invalid_client", "Invalid authorization credentials id").into(),
        );
    };
    let Some(client_secret) = credentials.next() else {
        return Err(bad_request_error(
            "invalid_client",
            "Invalid authorization credentials secret",
        )
        .into());
    };
    let client_secret_claims = decode_client_secret(decoding_key, client_secret)
        .context_bad_request("invalid_client", "Invalid client secret")?;
    if client_secret_claims.client_id != client_id {
        return Err(bad_request_error("invalid_client", "Invalid authorization client id").into());
    }
    if let Some(client_id) = &req.client_id
        && client_id != &client_secret_claims.client_id
    {
        return Err(
            bad_request_error("invalid_client", "Authenticated as a different client").into(),
        );
    }

    Ok(client_secret_claims.client_id)
}

/// Authenticate the client using client_secret_post auth.
/// i.e. using the the client_secret field.
fn authenticate_token_client_post(
    req: &TokenRequest,
    client_secret: &str,
    decoding_key: &DecodingKey,
) -> OauthResult<String> {
    tracing::debug!("Authenticating with client_secret_post client_secret parameter");

    let client_secret_claims = decode_client_secret(decoding_key, client_secret)
        .context_bad_request("invalid_client", "Invalid client secret")?;
    if let Some(client_id) = &req.client_id
        && client_id != &client_secret_claims.client_id
    {
        return Err(
            bad_request_error("invalid_client", "Authenticated as a different client").into(),
        );
    }
    Ok(client_secret_claims.client_id)
}

async fn validate_unauthenticated_client(req: &TokenRequest, store: &Store) -> OauthResult<String> {
    tracing::debug!("Unauthenticated client");

    let Some(client_id) = &req.client_id else {
        return Err(bad_request_error(
            "invalid_request",
            "Client id required for unauthenticated clients",
        )
        .into());
    };
    let Some(client) = store.get_client(client_id).await else {
        return Err(bad_request_error(
            "invalid_client",
            "Client is unregistered. Delete any dynamic clients and try again.",
        )
        .into());
    };
    if client.token_endpoint_auth_method != "none" {
        return Err(
            bad_request_error("invalid_client", "Client is requires authentication.").into(),
        );
    }

    Ok(client.client_id)
}

const CLIENT_SECRET_ISS: &str = "koso-mcp-oauth-client";
const CLIENT_SECRET_EXPIRY_SECS: u64 = 31 * 24 * 60 * 60;
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientSecretClaims {
    exp: u64,
    iss: String,
    client_id: String,
    client_name: String,
    expires_in: u64,
    scope: String,
    token_endpoint_auth_method: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
}

const AUTH_TOKEN_ISS: &str = "koso-mcp-oauth-auth";
const AUTH_TOKEN_EXPIRY_SECS: u64 = 8 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthTokenClaims {
    exp: u64,
    iss: String,
    client_id: String,
    session: String,
    expires_in: u64,
    scope: String,
    response_type: String,
    redirect_uri: String,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    user: User,
}

const ACCESS_TOKEN_ISS: &str = "koso-mcp-oauth-access";
const ACCESS_TOKEN_EXPIRY_SECS: u64 = 7 * 24 * 60 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    exp: u64,
    iss: String,
    client_id: String,
    expires_in: u64,
    scope: String,
    user: User,
    auth_token_claims: AuthTokenClaims,
}

const REFRESH_TOKEN_ISS: &str = "koso-mcp-oauth-refresh";
const REFRESH_TOKEN_EXPIRY_SECS: u64 = 15 * 24 * 60 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    exp: u64,
    iss: String,
    client_id: String,
    expires_in: u64,
    scope: String,
    user: User,
    auth_token_claims: AuthTokenClaims,
}

fn encode_client_secret(key: &EncodingKey, claims: &ClientSecretClaims) -> ApiResult<String> {
    encode_token(key, claims)
}

fn decode_client_secret(key: &DecodingKey, client_secret: &str) -> Result<ClientSecretClaims> {
    decode_token(key, client_secret, CLIENT_SECRET_ISS)
}

fn encode_auth_token(key: &EncodingKey, claims: &AuthTokenClaims) -> ApiResult<String> {
    encode_token(key, claims)
}

fn decode_auth_token(key: &DecodingKey, auth_token: &str) -> Result<AuthTokenClaims> {
    decode_token(key, auth_token, AUTH_TOKEN_ISS)
}

fn encode_access_token(key: &EncodingKey, claims: &AccessTokenClaims) -> ApiResult<String> {
    encode_token(key, claims)
}

fn decode_access_token(key: &DecodingKey, access_token: &str) -> Result<AccessTokenClaims> {
    decode_token(key, access_token, ACCESS_TOKEN_ISS)
}

fn encode_refresh_token(key: &EncodingKey, claims: &RefreshTokenClaims) -> ApiResult<String> {
    encode_token(key, claims)
}

fn decode_refresh_token(key: &DecodingKey, refresh_token: &str) -> Result<RefreshTokenClaims> {
    decode_token(key, refresh_token, REFRESH_TOKEN_ISS)
}

fn encode_token<T: Serialize>(key: &EncodingKey, claims: &T) -> ApiResult<String> {
    encode(&Header::default(), claims, key).context_status(
        StatusCode::INTERNAL_SERVER_ERROR,
        "server_error",
        "Something went wrong encoding token.",
    )
}

fn decode_token<T: DeserializeOwned>(key: &DecodingKey, token: &str, issuer: &str) -> Result<T> {
    let mut validation = Validation::default();
    validation.set_issuer(&[issuer]);
    validation.required_spec_claims.insert("iss".to_string());

    Ok(decode::<T>(token, key, &validation)
        .context("Invalid token")?
        .claims)
}

fn expires_at(expires_in: u64) -> ApiResult<u64> {
    let timer = SystemTime::now() + Duration::from_secs(expires_in);
    Ok(timer.duration_since(UNIX_EPOCH)?.as_secs())
}

/// Replace all empty strings with None.
fn trim_client_registration_request(
    mut req: ClientRegistrationRequest,
) -> ClientRegistrationRequest {
    swap_empty_with_none(&mut req.client_name);
    swap_empty_with_none(&mut req.scope);

    req
}

/// Replace all empty strings with None.
fn trim_authorize_request(mut req: AuthorizationDetailsRequest) -> AuthorizationDetailsRequest {
    swap_empty_with_none(&mut req.client_id);
    swap_empty_with_none(&mut req.redirect_uri);

    req
}

/// Replace all empty strings with None.
fn trim_approval_request(mut req: ApprovalRequest) -> ApprovalRequest {
    swap_empty_with_none(&mut req.client_id);
    swap_empty_with_none(&mut req.scope);
    swap_empty_with_none(&mut req.response_type);
    swap_empty_with_none(&mut req.code_challenge_method);
    swap_empty_with_none(&mut req.code_challenge);
    swap_empty_with_none(&mut req.redirect_uri);
    swap_empty_with_none(&mut req.resource);

    req
}

/// Replace all empty strings with None.
fn trim_token_request(mut req: TokenRequest) -> TokenRequest {
    swap_empty_with_none(&mut req.grant_type);
    swap_empty_with_none(&mut req.client_id);
    swap_empty_with_none(&mut req.client_secret);
    swap_empty_with_none(&mut req.code);
    swap_empty_with_none(&mut req.code_verifier);
    swap_empty_with_none(&mut req.refresh_token);
    swap_empty_with_none(&mut req.redirect_uri);
    swap_empty_with_none(&mut req.resource);

    req
}

/// If the options value is the empty string, replace it with None.
/// Implements https://datatracker.ietf.org/doc/html/rfc6749
/// >  Parameters sent without a value
/// >  MUST be treated as if they were omitted from the request.
fn swap_empty_with_none(s: &mut Option<String>) {
    if let Some(ss) = s
        && ss.is_empty()
    {
        s.take();
    }
}

impl std::fmt::Debug for ClientRegistrationResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientRegistrationResponse")
            .field("client_id", &self.client_id)
            .field("client_name", &self.client_name)
            .field("scope", &self.scope)
            .field(
                "token_endpoint_auth_method",
                &self.token_endpoint_auth_method,
            )
            .field("redirect_uris", &self.redirect_uris)
            .field("grant_types", &self.grant_types)
            .field("response_types", &self.response_types)
            .field("client_secret_expires_at", &self.client_secret_expires_at)
            .field("client_secret", &"<REDACTED>")
            .finish()
    }
}

impl std::fmt::Debug for TokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenRequest")
            .field("grant_type", &self.grant_type)
            .field("client_id", &self.client_id)
            .field(
                "client_secret",
                &self.client_secret.as_ref().map(|_| "<REDACTED>"),
            )
            .field("resource", &self.resource)
            .field("redirect_uri", &self.redirect_uri)
            .field("code", &self.code.as_ref().map(|_| "<REDACTED>"))
            .field(
                "code_verifier",
                &self.code_verifier.as_ref().map(|_| "<REDACTED>"),
            )
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "<REDACTED>"),
            )
            .field("other", &self.other)
            .finish()
    }
}

impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenRequest")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field("scope", &self.scope)
            .field("access_token", &"<REDACTED>")
            .field("refresh_token", &"<REDACTED>")
            .finish()
    }
}

/// Apply cache controls per https://datatracker.ietf.org/doc/html/rfc6749#section-5.1.
async fn set_cache_control(request: extract::Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("Pragma", HeaderValue::from_static("no-cache"));

    response
}

pub(crate) type OauthResult<T> = Result<T, OauthErrorResponse>;

#[derive(Debug)]
pub(crate) struct OauthErrorResponse {
    status: StatusCode,
    /// https://datatracker.ietf.org/doc/html/rfc6749#section-5.2
    /// invalid_request, invalid_client, invalid_grant, unauthorized_client, unsupported_grant_type, invalid_scope
    error: String,
    error_description: String,
}

#[derive(serde::Serialize)]
struct OauthErrorResponseBody {
    error: String,
    error_description: String,
}

/// Converts from OauthErrorResponse to Response.
/// Creates error bodies following: https://www.rfc-editor.org/rfc/rfc6749#section-5.2
impl IntoResponse for OauthErrorResponse {
    fn into_response(self) -> Response {
        let body = axum::Json(OauthErrorResponseBody {
            error: self.error,
            error_description: self.error_description,
        });

        let mut res = (self.status, body).into_response();
        if self.status == StatusCode::UNAUTHORIZED {
            // Append the WWW-Authenticate header so the client knows how to proceed.
            if let Err(err) = add_www_authenticate_header(&mut res) {
                tracing::error!("Failed to crate authenticate header value: ${err:#}");
            }
        }
        res
    }
}

fn add_www_authenticate_header(res: &mut Response) -> Result<()> {
    res.headers_mut().insert(
        "WWW-Authenticate",
        HeaderValue::from_str(&format!(
            "Bearer resource_metadata={}/.well-known/oauth-protected-resource/api/mcp/sse",
            settings().host
        ))
        .context("Failed to construct www-authenticate header value")?,
    );
    Ok(())
}

impl From<ErrorResponse> for OauthErrorResponse {
    fn from(err: ErrorResponse) -> Self {
        let detail = err.details.first();
        OauthErrorResponse {
            status: err.status,
            error: detail
                .map(|d| d.reason)
                .unwrap_or("server_error")
                .to_string(),
            error_description: detail
                .map(|d| d.msg.as_str())
                .unwrap_or("Internal error, something went wrong")
                .to_string(),
        }
    }
}
