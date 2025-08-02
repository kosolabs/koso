/// Oauth authorization for MCP. The sequence diagram at
/// https://modelcontextprotocol.io/specification/draft/basic/authorization#authorization-flow-steps
/// is a useful resource.
use crate::{
    api::{
        self, ApiResult, IntoApiResult, bad_request_error, error_response,
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
use sqlx::PgPool;
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tower_http::cors::{self, CorsLayer};
use uuid::Uuid;

/// Implements MCP oauth: https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization
pub(crate) fn router(pool: &'static PgPool) -> Result<Router> {
    // https://datatracker.ietf.org/doc/html/rfc9700#name-authorization-code-grant:~:text=Cross%2DOrigin%20Resource%20Sharing
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
                .route(
                    "/authorization_details",
                    post(oauth_authorization_details)
                        .layer(middleware::from_fn(google::authenticate)),
                )
                .route(
                    "/approve",
                    post(oauth_approve)
                        .options(oauth_approve)
                        .layer(middleware::from_fn(google::authenticate)),
                )
                .layer((
                    Extension(Store::new(pool)),
                    middleware::from_fn(set_cache_control),
                ))
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
) -> Response<Body> {
    let access_token_claims: AccessTokenClaims = match _authenticate(&decoding_key, &mut req)
        .context_status(
            StatusCode::UNAUTHORIZED,
            // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#name-error-codes
            "invalid_token",
            "Unauthenticated user",
        ) {
        Ok(access_token_claims) => access_token_claims,
        Err(err) => {
            let mut res = err.into_response();
            // Append the WWW-Authenticate header so the client knows how to proceed.
            if let Err(err) = add_www_authenticate_header(&mut res) {
                return err.into_response();
            }
            return res;
        }
    };

    let mut user = access_token_claims.user;
    user.email = user.email.to_lowercase();

    tracing::Span::current().record("email", user.email.clone());
    assert!(req.extensions_mut().insert(user).is_none());

    next.run(req).await
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
        parts[1].into()
    };

    // Decode the access token.
    decode_access_token(decoding_key, &access_token)
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
#[derive(Debug, Serialize)]
struct ResourceServerMetadata {
    /// https://www.rfc-editor.org/rfc/rfc8707.html
    resource: String,
    authorization_servers: Vec<String>,
    bearer_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

/// https://datatracker.ietf.org/doc/rfc9728/
#[tracing::instrument()]
async fn get_resource_server_metadata() -> ApiResult<Json<ResourceServerMetadata>> {
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
#[derive(Debug, Serialize)]
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
async fn get_authorization_server_metadata() -> ApiResult<Json<AuthorizationServerMetadata>> {
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
    pool: &'static PgPool,
    tokens: Arc<Mutex<HashMap<String, AuthTokenMetadata>>>,
}

impl Store {
    fn new(pool: &'static PgPool) -> Store {
        Store {
            pool,
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

type ClientRow = (sqlx::types::Json<ClientMetadata>,);

#[derive(Deserialize, Serialize)]
struct ClientMetadata {
    client_id: String,
    client_name: String,
    registered_at: u64,
    client_secret_expires_at: u64,
    scope: String,
    token_endpoint_auth_method: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
}

#[derive(Clone, Debug)]
struct AuthTokenMetadata {
    issued_at: u64,
    expires_at: u64,
    client_id: String,
    token_id: String,
    scope: String,
    #[allow(dead_code)]
    response_type: String,
    redirect_uri: String,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    user: User,
}

impl Store {
    async fn insert_client(&self, client_metadata: &ClientMetadata) -> ApiResult<()> {
        let res = sqlx::query(
            "
                INSERT INTO oauth_clients (client_id, client_metadata)
                VALUES ($1, $2);",
        )
        .bind(&client_metadata.client_id)
        .bind(sqlx::types::Json(&client_metadata))
        .execute(self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(anyhow!("Client already exists: {}", client_metadata.client_id).into());
        }

        Ok(())
    }

    async fn get_client(&self, client_id: &str) -> ApiResult<Option<ClientMetadata>> {
        let client: Option<ClientRow> = sqlx::query_as(
            "
            SELECT client_metadata
            FROM oauth_clients
            WHERE client_id=$1",
        )
        .bind(client_id)
        .fetch_optional(self.pool)
        .await
        .context(format!("Failed to get client {client_id}"))?;
        if let Some((client,)) = client {
            Ok(Some(client.0))
        } else {
            Ok(None)
        }
    }

    async fn insert_auth_token(&self, auth_token: &AuthTokenMetadata) -> ApiResult<()> {
        let mut tokens = self.tokens.lock().await;
        // Drop all tokens to avoid an OOM. The limit is sufficiently high such
        // we're unlikely to hit it between server restarts.
        if tokens.len() > 2500 {
            tracing::error!("Cleared oauth tokens. Consider implementing better eviction.");
            tokens.clear();
            tokens.shrink_to_fit();
        }
        match tokens.entry(auth_token.token_id.clone()) {
            Entry::Occupied(entry) => {
                return Err(anyhow!("Token already exists: {}", entry.key()).into());
            }
            Entry::Vacant(entry) => {
                entry.insert(auth_token.clone());
            }
        }

        Ok(())
    }

    async fn consume_auth_token(&self, token_id: &str) -> Result<AuthTokenMetadata> {
        self.tokens
            .lock()
            .await
            .remove(token_id)
            .with_context(|| format!("Token {token_id} not found"))
    }
}

/// Note: All fields must be optional. Validation should occur with the handler.
/// See `swap_empty_with_none` below.
#[derive(Debug, Deserialize)]
struct ClientRegistrationRequest {
    client_name: Option<String>,
    scope: Option<String>,
    #[serde(default)]
    redirect_uris: Vec<String>,
    token_endpoint_auth_method: Option<String>,
    #[serde(default)]
    grant_types: Vec<String>,
    #[serde(default)]
    response_types: Vec<String>,
    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ClientRegistrationResponse {
    client_id: String,
    client_name: String,
    scope: String,
    token_endpoint_auth_method: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    client_secret_expires_at: u64,
    client_secret: Secret,
}

// Handle dynamic client registration
// https://datatracker.ietf.org/doc/html/rfc7591#section-3.1
#[tracing::instrument(skip(key, store, req))]
async fn oauth_register(
    Extension(key): Extension<EncodingKey>,
    Extension(store): Extension<Store>,
    Json(req): Json<ClientRegistrationRequest>,
) -> ApiResult<Json<ClientRegistrationResponse>> {
    let req = trim_client_registration_request(req);
    tracing::debug!("Registering client: {req:?}");

    let client_metadata = create_client(req).await?;

    // Generate the client secret.
    let client_secret = encode_client_secret(
        &key,
        &ClientSecretClaims {
            iat: client_metadata.registered_at,
            exp: client_metadata.client_secret_expires_at,
            iss: CLIENT_SECRET_ISS.to_string(),
            client_id: client_metadata.client_id.clone(),
        },
    )?;

    // Persist the client.
    store.insert_client(&client_metadata).await?;

    // Create the response.
    let response = ClientRegistrationResponse {
        client_id: client_metadata.client_id,
        client_name: client_metadata.client_name,
        scope: client_metadata.scope,
        token_endpoint_auth_method: client_metadata.token_endpoint_auth_method,
        redirect_uris: client_metadata.redirect_uris,
        grant_types: client_metadata.grant_types,
        response_types: client_metadata.response_types,
        client_secret_expires_at: client_metadata.client_secret_expires_at,
        client_secret,
    };
    tracing::debug!("Registered client: {response:?}");

    Ok(Json(response))
}

async fn create_client(req: ClientRegistrationRequest) -> ApiResult<ClientMetadata> {
    // Validate the request
    let redirect_uris = validate_redirect_uris(req.redirect_uris)?;
    let response_types = req.response_types;
    if !response_types.is_empty() && !response_types.contains(&CODE_RESPONSE_TYPE.to_string()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only the 'code' response_type is supported",
        ));
    }
    let grant_types = req.grant_types;
    if !grant_types.is_empty() && !grant_types.contains(&CODE_GRANT_TYPE.to_string()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "grant_types must contain 'authorization_code'",
        ));
    }
    let scope = req.scope.unwrap_or_else(|| READ_WRITE_SCOPE.to_string());
    if scope != READ_WRITE_SCOPE {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only read_write scope is supported",
        ));
    }
    let token_endpoint_auth_method = req
        .token_endpoint_auth_method
        .unwrap_or_else(|| BASIC_AUTH_METHOD.to_string());
    if !AUTH_METHODS_SUPPORTED.contains(&token_endpoint_auth_method.as_str()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only client_secret_basic, client_secret_post and none token_auth_method are supported",
        ));
    }

    let client_id = format!("client-{}", Uuid::new_v4());
    let mut client_name = req.client_name.unwrap_or_else(|| client_id.clone());
    if client_name.len() > 255 {
        client_name.truncate(255);
        client_name = format!("{client_name}..");
    }

    Ok(ClientMetadata {
        client_id,
        client_name,
        registered_at: now()?,
        client_secret_expires_at: expires_at(CLIENT_SECRET_EXPIRY_SECS)?,
        scope,
        token_endpoint_auth_method,
        redirect_uris,
        grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
        response_types: vec![CODE_RESPONSE_TYPE.to_string()],
    })
}

fn validate_redirect_uris(redirect_uris: Vec<String>) -> ApiResult<Vec<String>> {
    if redirect_uris.is_empty() {
        return Err(bad_request_error(
            "invalid_redirect_uri",
            "At least one redirect uri is required",
        ));
    }
    if redirect_uris.len() > 15 {
        return Err(bad_request_error(
            "invalid_redirect_uri",
            "Too many redirect uris",
        ));
    }
    if redirect_uris.iter().any(|s| s.is_empty()) {
        return Err(bad_request_error(
            "invalid_redirect_uri",
            "Redirect uri cannot be empty",
        ));
    }
    if redirect_uris.iter().any(|s| s.len() > 2000) {
        return Err(bad_request_error(
            "invalid_redirect_uri",
            "Redirect uri too long",
        ));
    }
    Ok(redirect_uris)
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
    let Some(client_metadata) = store.get_client(&client_id).await? else {
        return Err(bad_request_error(
            "unauthorized_client",
            "Unregistered client. Clear any auth state, delete dynamic clients, and try again.",
        ));
    };
    validate_redirect_uri(&client_metadata.redirect_uris, &redirect_uri)?;

    Ok(Json(AuthorizationDetailsResponse {
        client_name: client_metadata.client_name,
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
    code: Secret,
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

    let auth_token_metadata = issue_auth_token(req, user, &store).await?;

    // Encode the token.
    let auth_token = encode_auth_token(
        &encoding_key,
        &AuthTokenClaims {
            iat: auth_token_metadata.issued_at,
            exp: auth_token_metadata.expires_at,
            iss: AUTH_TOKEN_ISS.to_string(),
            client_id: auth_token_metadata.client_id.clone(),
            token_id: auth_token_metadata.token_id.clone(),
        },
    )?;

    // Persist the token.
    store.insert_auth_token(&auth_token_metadata).await?;

    tracing::info!("Approved authorization, created auth token: {auth_token_metadata:?}");

    Ok(Json(ApprovalResponse { code: auth_token }))
}

async fn issue_auth_token(
    req: ApprovalRequest,
    user: User,
    store: &Store,
) -> ApiResult<AuthTokenMetadata> {
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
    let Some(client_metadata) = store.get_client(&client_id).await? else {
        return Err(bad_request_error(
            "unauthorized_client",
            "Unregistered client. Clear any auth state, delete dynamic clients, and try again.",
        ));
    };
    validate_redirect_uri(&client_metadata.redirect_uris, &redirect_uri)?;

    // Encode the auth token
    Ok(AuthTokenMetadata {
        issued_at: now()?,
        expires_at: expires_at(AUTH_TOKEN_EXPIRY_SECS)?,
        client_id,
        token_id: format!("token-{}", Uuid::new_v4()),
        scope,
        response_type,
        redirect_uri,
        code_challenge,
        code_challenge_method,
        user,
    })
}

/// Note: All fields must be optional. Validation should occur with the handler.
/// See `swap_empty_with_none` below.
#[derive(Clone, Debug, Deserialize)]
struct TokenRequest {
    /// refresh_token or authorization_code
    grant_type: Option<String>,
    client_id: Option<String>,
    client_secret: Option<Secret>,
    scope: Option<String>,
    /// https://www.rfc-editor.org/rfc/rfc8707.html#name-resource-parameter
    resource: Option<String>,
    redirect_uri: Option<String>,

    // authorization_code fields
    code: Option<Secret>,
    code_verifier: Option<Secret>,

    // refresh_token_fields
    refresh_token: Option<Secret>,

    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    token_type: String,
    expires_in: u64,
    scope: String,
    access_token: Secret,
    refresh_token: Secret,
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
) -> Response<Body> {
    match _oauth_token(store, decoding_key, encoding_key, headers, req).await {
        Ok(res) => res.into_response(),
        Err(err) => {
            let mut res = err.into_response();
            if res.status() == StatusCode::UNAUTHORIZED {
                // Append the WWW-Authenticate header so the client knows how to proceed.
                // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#section-3.2.4
                if let Err(err) = add_www_authenticate_header(&mut res) {
                    return err.into_response();
                }
            }
            return res;
        }
    }
}

async fn _oauth_token(
    store: Store,
    decoding_key: DecodingKey,
    encoding_key: EncodingKey,
    headers: HeaderMap,
    req: TokenRequest,
) -> ApiResult<Json<TokenResponse>> {
    let req = trim_token_request(req);
    tracing::info!("Handling token request: {req:?}");

    // Issue tokens based on the given grant type.
    let (access_claims, refresh_claims) = match req.grant_type.as_deref() {
        Some(CODE_GRANT_TYPE) => {
            issue_from_authorization_code(store, decoding_key, headers, req).await
        }
        Some(REFRESH_GRANT_TYPE) => {
            issue_from_refresh_token(store, decoding_key, headers, req).await
        }
        Some(_) => Err(bad_request_error(
            "unsupported_grant_type",
            "Only authorization_code is supported",
        )),
        None => Err(bad_request_error("invalid_request", "grant_type required")),
    }?;
    let access_token = encode_access_token(&encoding_key, &access_claims)?;
    let refresh_token = encode_refresh_token(&encoding_key, &refresh_claims)?;

    tracing::info!("Created access token: {access_claims:?}, refresh token: {refresh_claims:?}");

    Ok(Json(TokenResponse {
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_EXPIRY_SECS,
        scope: access_claims.scope,
        access_token,
        refresh_token,
    }))
}

async fn issue_from_authorization_code(
    store: Store,
    decoding_key: DecodingKey,
    headers: HeaderMap,
    req: TokenRequest,
) -> ApiResult<(AccessTokenClaims, RefreshTokenClaims)> {
    // Authenticate the client or allow unauthenticated clients.
    let client_metadata = authenticate_token_client(&headers, &req, &store, &decoding_key).await?;

    // Validate the authorization token.
    let (auth_token_claims, auth_token) =
        validate_authorization_code(&req, &client_metadata, &store, &decoding_key).await?;

    // Create the token claims.
    let access_claims = AccessTokenClaims {
        iat: now()?,
        exp: expires_at(ACCESS_TOKEN_EXPIRY_SECS)?,
        iss: ACCESS_TOKEN_ISS.to_string(),
        scope: auth_token.scope,
        user: auth_token.user,
        auth_token_claims,
    };
    let refresh_claims = RefreshTokenClaims {
        iat: access_claims.iat,
        exp: expires_at(REFRESH_TOKEN_EXPIRY_SECS)?,
        iss: REFRESH_TOKEN_ISS.to_string(),
        scope: access_claims.scope.clone(),
        user: access_claims.user.clone(),
        auth_token_claims: access_claims.auth_token_claims.clone(),
    };
    Ok((access_claims, refresh_claims))
}

async fn validate_authorization_code(
    req: &TokenRequest,
    client_metadata: &ClientMetadata,
    store: &Store,
    decoding_key: &DecodingKey,
) -> ApiResult<(AuthTokenClaims, AuthTokenMetadata)> {
    // Decode the auth token.
    let Some(code) = &req.code else {
        return Err(bad_request_error(
            "invalid_request",
            "code required for authorization_code grant type",
        ));
    };
    let auth_token_claims = decode_auth_token(decoding_key, code)?;
    let auth_token = store
        .consume_auth_token(&auth_token_claims.token_id)
        .await
        .context_bad_request("invalid_grant", "Auth code already used")?;
    if auth_token.client_id != auth_token_claims.client_id {
        return Err(bad_request_error("invalid_grant", "Invalid token client"));
    }

    // PKCE: Verify the challenge against the verifier.
    validate_code_challenge(req, &auth_token)?;

    // Validate the token was issued for this client
    // The check is optional as client_id may be omitted for authenticated auth_token requests.
    if let Some(client_id) = &req.client_id
        && client_id != &auth_token.client_id
    {
        return Err(bad_request_error(
            "invalid_grant",
            "Auth code issued to another client",
        ));
    }
    if client_metadata.client_id != auth_token_claims.client_id {
        return Err(bad_request_error(
            "invalid_grant",
            "Grant issued for another client",
        ));
    }

    if let Some(scope) = &req.scope
        && scope != &auth_token.scope
    {
        return Err(bad_request_error("invalid_scope", "Mismatched scope"));
    }

    // This is only for backwards compatibility with OAuth 2.0
    // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#name-redirect-uri-parameter-in-t
    if let Some(redirect_uri) = &req.redirect_uri
        && &auth_token.redirect_uri != redirect_uri
    {
        return Err(bad_request_error(
            "invalid_request",
            "redirect_uri doesn't match the one in the authorization request",
        ));
    }

    Ok((auth_token_claims, auth_token))
}

/// Implement PKCE.
/// See https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#section-4.1.1
fn validate_code_challenge(req: &TokenRequest, auth_token: &AuthTokenMetadata) -> ApiResult<()> {
    match (
        &auth_token.code_challenge_method,
        &auth_token.code_challenge,
        &req.code_verifier,
    ) {
        (Some(method), Some(challenge), Some(verifier)) => {
            if method != S256_CHALLENGE_METHOD {
                return Err(bad_request_error("invalid_grant", "Only S256 is supported"));
            }
            let actual_challenge = BASE64_URL_SAFE_NO_PAD
                .encode(Sha256::new().chain_update(verifier.data()).finalize());
            if &actual_challenge != challenge {
                return Err(bad_request_error(
                    "invalid_grant",
                    "Challenge does not match verifier",
                ));
            }
        }
        (None, None, None) => {}
        // Prevent PKCE downgrade attacks.
        // https://datatracker.ietf.org/doc/html/rfc9700#name-authorization-code-grant:~:text=servers%20MUST%20mitigate-,PKCE%20downgrade%20attacks,-by%20ensuring%20that
        _ => {
            return Err(bad_request_error(
                "invalid_grant",
                "Method, challenge and verifier must all be set or unset",
            ));
        }
    }

    Ok(())
}

async fn issue_from_refresh_token(
    store: Store,
    decoding_key: DecodingKey,
    headers: HeaderMap,
    req: TokenRequest,
) -> ApiResult<(AccessTokenClaims, RefreshTokenClaims)> {
    // Authenticate the client or allow unauthenticated clients.
    let client_metadata = authenticate_token_client(&headers, &req, &store, &decoding_key).await?;

    // Validate the refresh token.
    let req_refresh_claims = validate_refresh_token(&req, &client_metadata, &decoding_key)?;

    // Create the token claims.
    let access_claims = AccessTokenClaims {
        iat: now()?,
        exp: expires_at(ACCESS_TOKEN_EXPIRY_SECS)?,
        iss: ACCESS_TOKEN_ISS.to_string(),
        scope: req_refresh_claims.scope,
        user: req_refresh_claims.user,
        auth_token_claims: req_refresh_claims.auth_token_claims,
    };
    let refresh_claims = RefreshTokenClaims {
        iat: access_claims.iat,
        // Limit the lifetime of the new refresh token to that of the existing one.
        exp: req_refresh_claims.exp,
        iss: REFRESH_TOKEN_ISS.to_string(),
        scope: access_claims.scope.clone(),
        user: access_claims.user.clone(),
        auth_token_claims: access_claims.auth_token_claims.clone(),
    };
    Ok((access_claims, refresh_claims))
}

fn validate_refresh_token(
    req: &TokenRequest,
    client_metadata: &ClientMetadata,
    decoding_key: &DecodingKey,
) -> ApiResult<RefreshTokenClaims> {
    // Decode the refresh token and grab the auth token.
    let Some(refresh_token) = &req.refresh_token else {
        return Err(bad_request_error(
            "invalid_request",
            "refresh_token required for refresh_token grant type",
        ));
    };
    let refresh_token = decode_refresh_token(decoding_key, refresh_token)?;

    // Validate the token was issued for this client
    // The check is optional as client_id may be omitted for refresh_token requests.
    if let Some(client_id) = &req.client_id
        && client_id != &refresh_token.auth_token_claims.client_id
    {
        return Err(bad_request_error(
            "invalid_grant",
            "Refresh token issued to another client",
        ));
    }
    if client_metadata.client_id != refresh_token.auth_token_claims.client_id {
        return Err(bad_request_error(
            "invalid_grant",
            "Grant issued for another client",
        ));
    }

    if let Some(scope) = &req.scope
        && scope != &refresh_token.scope
    {
        return Err(bad_request_error("invalid_scope", "Mismatched scope"));
    }

    Ok(refresh_token)
}

async fn authenticate_token_client(
    headers: &HeaderMap,
    req: &TokenRequest,
    store: &Store,
    decoding_key: &DecodingKey,
) -> ApiResult<ClientMetadata> {
    match (headers.get("Authorization"), &req.client_secret) {
        // client_secret_basic
        // Prefer to use the Authorization header.
        (Some(header), _) => {
            authenticate_token_client_basic(req, header, store, decoding_key).await
        }

        // client_secret_post
        // Use the request body when the Authorization header is absent.
        (None, Some(client_secret)) => {
            authenticate_token_client_post(req, client_secret, store, decoding_key).await
        }

        // none
        // Unauthenticated clients.
        (None, None) => validate_unauthenticated_client(req, store).await,
    }
}

/// Authenticate the client using client_secret_basic auth.
/// i.e. Authorization header Basic
async fn authenticate_token_client_basic(
    req: &TokenRequest,
    header: &HeaderValue,
    store: &Store,
    decoding_key: &DecodingKey,
) -> ApiResult<ClientMetadata> {
    tracing::debug!("Authenticating with client_secret_basic Authorization header");

    let Some(credentials) = header.as_bytes().strip_prefix(b"Basic ") else {
        return Err(bad_request_error(
            "invalid_request",
            "Invalid authorization header",
        ));
    };
    let credentials: String = BASE64_STANDARD
        .decode(credentials)
        .context_bad_request("invalid_request", "Invalid authorization credentials")?
        .try_into()
        .context_bad_request("invalid_request", "Invalid authorization credentials")?;
    let mut credentials = credentials.split(":");
    let Some(client_id) = credentials.next() else {
        return Err(bad_request_error(
            "invalid_request",
            "Invalid authorization credentials id",
        ));
    };
    let Some(client_secret) = credentials.next().map(Into::into) else {
        return Err(bad_request_error(
            "invalid_request",
            "Invalid authorization credentials secret",
        ));
    };
    let client_secret_claims = decode_client_secret(decoding_key, &client_secret)?;
    if client_secret_claims.client_id != client_id {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Invalid authorization client id",
            None,
        ));
    }
    if let Some(client_id) = &req.client_id
        && client_id != &client_secret_claims.client_id
    {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Authenticated as a different client",
            None,
        ));
    }

    let Some(client_metadata) = store.get_client(&client_secret_claims.client_id).await? else {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Unregistered client. Clear any auth state, delete dynamic clients, and try again.",
            None,
        ));
    };
    Ok(client_metadata)
}

/// Authenticate the client using client_secret_post auth.
/// i.e. using the the client_secret field.
async fn authenticate_token_client_post(
    req: &TokenRequest,
    client_secret: &Secret,
    store: &Store,
    decoding_key: &DecodingKey,
) -> ApiResult<ClientMetadata> {
    tracing::debug!("Authenticating with client_secret_post client_secret parameter");

    let client_secret_claims = decode_client_secret(decoding_key, client_secret)?;
    if let Some(client_id) = &req.client_id
        && client_id != &client_secret_claims.client_id
    {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Authenticated as a different client",
            None,
        ));
    }

    let Some(client_metadata) = store.get_client(&client_secret_claims.client_id).await? else {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Unregistered client. Clear any auth state, delete dynamic clients, and try again.",
            None,
        ));
    };
    Ok(client_metadata)
}

async fn validate_unauthenticated_client(
    req: &TokenRequest,
    store: &Store,
) -> ApiResult<ClientMetadata> {
    tracing::debug!("Unauthenticated client");

    let Some(client_id) = &req.client_id else {
        return Err(bad_request_error(
            "invalid_request",
            "Client id required for unauthenticated clients",
        ));
    };
    let Some(client_metadata) = store.get_client(client_id).await? else {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Unregistered client. Clear any auth state, delete dynamic clients, and try again.",
            None,
        ));
    };
    if client_metadata.token_endpoint_auth_method != "none" {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Confidential client requires authentication.",
            None,
        ));
    }

    Ok(client_metadata)
}

fn validate_redirect_uri(valid_redirect_uris: &[String], redirect_uri: &String) -> ApiResult<()> {
    // TODO: ignore ports for localhost.
    // https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-13#section-4.1.1
    // > The only exception is native apps using a localhost URI: In this case, the
    // > authorization server MUST allow variable port numbers as described in Section 7.3 of [RFC8252].
    if !valid_redirect_uris.contains(redirect_uri) {
        return Err(bad_request_error(
            "invalid_request",
            "Registered redirect uri doesn't match the provided. Danger!",
        ));
    }
    Ok(())
}

const CLIENT_SECRET_ISS: &str = "koso-mcp-oauth-client";
const CLIENT_SECRET_EXPIRY_SECS: u64 = 91 * 24 * 60 * 60;
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientSecretClaims {
    iat: u64,
    exp: u64,
    iss: String,
    client_id: String,
}

const AUTH_TOKEN_ISS: &str = "koso-mcp-oauth-auth";
const AUTH_TOKEN_EXPIRY_SECS: u64 = 2 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthTokenClaims {
    iat: u64,
    exp: u64,
    iss: String,
    client_id: String,
    token_id: String,
}

const ACCESS_TOKEN_ISS: &str = "koso-mcp-oauth-access";
const ACCESS_TOKEN_EXPIRY_SECS: u64 = 7 * 24 * 60 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    iat: u64,
    exp: u64,
    iss: String,
    scope: String,
    user: User,
    auth_token_claims: AuthTokenClaims,
}

const REFRESH_TOKEN_ISS: &str = "koso-mcp-oauth-refresh";
const REFRESH_TOKEN_EXPIRY_SECS: u64 = 90 * 24 * 60 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    iat: u64,
    exp: u64,
    iss: String,
    scope: String,
    user: User,
    auth_token_claims: AuthTokenClaims,
}

fn encode_client_secret(key: &EncodingKey, claims: &ClientSecretClaims) -> ApiResult<Secret> {
    encode_token(key, claims)
}

fn decode_client_secret(
    key: &DecodingKey,
    client_secret: &Secret,
) -> ApiResult<ClientSecretClaims> {
    decode_token(key, client_secret, CLIENT_SECRET_ISS).context_status(
        StatusCode::UNAUTHORIZED,
        "invalid_client",
        "Invalid client secret",
    )
}

fn encode_auth_token(key: &EncodingKey, claims: &AuthTokenClaims) -> ApiResult<Secret> {
    encode_token(key, claims)
}

fn decode_auth_token(key: &DecodingKey, auth_token: &Secret) -> ApiResult<AuthTokenClaims> {
    decode_token(key, auth_token, AUTH_TOKEN_ISS)
        .context_bad_request("invalid_grant", "Invalid auth token")
}

fn encode_access_token(key: &EncodingKey, claims: &AccessTokenClaims) -> ApiResult<Secret> {
    encode_token(key, claims)
}

fn decode_access_token(key: &DecodingKey, access_token: &Secret) -> Result<AccessTokenClaims> {
    decode_token(key, access_token, ACCESS_TOKEN_ISS)
}

fn encode_refresh_token(key: &EncodingKey, claims: &RefreshTokenClaims) -> ApiResult<Secret> {
    encode_token(key, claims)
}

fn decode_refresh_token(
    key: &DecodingKey,
    refresh_token: &Secret,
) -> ApiResult<RefreshTokenClaims> {
    decode_token(key, refresh_token, REFRESH_TOKEN_ISS)
        .context_bad_request("invalid_grant", "Invalid refresh token")
}

fn encode_token<T: Serialize>(key: &EncodingKey, claims: &T) -> ApiResult<Secret> {
    Ok(encode(&Header::default(), claims, key)
        .context_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Something went wrong encoding token.",
        )?
        .into())
}

fn decode_token<T: DeserializeOwned>(key: &DecodingKey, token: &Secret, issuer: &str) -> Result<T> {
    let mut validation = Validation::default();
    validation.set_issuer(&[issuer]);
    validation.required_spec_claims.insert("iss".to_string());

    Ok(decode::<T>(token.data(), key, &validation)
        .context("Invalid token")?
        .claims)
}

fn expires_at(expires_in: u64) -> ApiResult<u64> {
    let timer = SystemTime::now() + Duration::from_secs(expires_in);
    Ok(timer.duration_since(UNIX_EPOCH)?.as_secs())
}

fn now() -> ApiResult<u64> {
    let timer = SystemTime::now();
    Ok(timer.duration_since(UNIX_EPOCH)?.as_secs())
}

/// Wrapper that makes it hard to accidentally log a secret.
#[derive(Clone, Serialize, Deserialize)]
struct Secret(String);

impl<S: Into<String>> From<S> for Secret {
    fn from(data: S) -> Self {
        Secret(data.into())
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<REDACTED>")
    }
}

impl Secret {
    fn data(&self) -> &str {
        &self.0
    }
}

/// Replace all empty strings with None.
fn trim_client_registration_request(req: ClientRegistrationRequest) -> ClientRegistrationRequest {
    ClientRegistrationRequest {
        client_name: trim_to_none(req.client_name),
        scope: trim_to_none(req.scope),
        redirect_uris: req.redirect_uris,
        token_endpoint_auth_method: trim_to_none(req.token_endpoint_auth_method),
        grant_types: req.grant_types,
        response_types: req.response_types,
        other: req.other,
    }
}

/// Replace all empty strings with None.
fn trim_authorize_request(req: AuthorizationDetailsRequest) -> AuthorizationDetailsRequest {
    AuthorizationDetailsRequest {
        client_id: trim_to_none(req.client_id),
        redirect_uri: trim_to_none(req.redirect_uri),
        other: req.other,
    }
}

/// Replace all empty strings with None.
fn trim_approval_request(req: ApprovalRequest) -> ApprovalRequest {
    ApprovalRequest {
        client_id: trim_to_none(req.client_id),
        scope: trim_to_none(req.scope),
        response_type: trim_to_none(req.response_type),
        code_challenge_method: trim_to_none(req.code_challenge_method),
        code_challenge: trim_to_none(req.code_challenge),
        redirect_uri: trim_to_none(req.redirect_uri),
        resource: trim_to_none(req.resource),
        other: req.other,
    }
}

/// Replace all empty strings with None.
fn trim_token_request(req: TokenRequest) -> TokenRequest {
    TokenRequest {
        grant_type: trim_to_none(req.grant_type),
        client_id: trim_to_none(req.client_id),
        client_secret: trim_secret_to_none(req.client_secret),
        scope: trim_to_none(req.scope),
        resource: trim_to_none(req.resource),
        redirect_uri: trim_to_none(req.redirect_uri),
        code: trim_secret_to_none(req.code),
        code_verifier: trim_secret_to_none(req.code_verifier),
        refresh_token: trim_secret_to_none(req.refresh_token),
        other: req.other,
    }
}

/// If the options value is the empty string, return None.
/// Implements https://datatracker.ietf.org/doc/html/rfc6749
/// >  Parameters sent without a value
/// >  MUST be treated as if they were omitted from the request.
fn trim_to_none(s: Option<String>) -> Option<String> {
    if let Some(ss) = &s
        && ss.is_empty()
    {
        None
    } else {
        s
    }
}

fn trim_secret_to_none(s: Option<Secret>) -> Option<Secret> {
    if let Some(ss) = &s
        && ss.data().is_empty()
    {
        None
    } else {
        s
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

fn add_www_authenticate_header(res: &mut Response) -> ApiResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::Value;

    #[test_log::test(sqlx::test)]
    async fn test_oauth_register_full(pool: PgPool) {
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);
        let store = Store::new(Box::leak(Box::new(pool)));

        // Register the cliet
        let redirect_uri1: String = "http:://localhost/test/redirect1".to_string();
        let redirect_uri2: String = "http:://localhost/test/redirect2".to_string();
        let req = ClientRegistrationRequest {
            client_name: Some("Client 123".to_string()),
            scope: Some(READ_WRITE_SCOPE.to_string()),
            token_endpoint_auth_method: Some(BASIC_AUTH_METHOD.to_string()),
            redirect_uris: vec![redirect_uri1.clone(), redirect_uri2.clone()],
            grant_types: vec![CODE_GRANT_TYPE.to_string(), "ignored_grant".to_string()],
            response_types: vec![
                CODE_RESPONSE_TYPE.to_string(),
                "ignored_response_type".to_string(),
            ],
            other: Value::Null,
        };
        let Json(res) = oauth_register(Extension(key), Extension(store.clone()), Json(req))
            .await
            .unwrap();

        // Asserts over the response
        assert!(!res.client_id.is_empty());
        assert_eq!(res.client_name, "Client 123");
        assert_eq!(res.scope, READ_WRITE_SCOPE);
        assert_eq!(res.token_endpoint_auth_method, BASIC_AUTH_METHOD);
        assert_eq!(
            res.redirect_uris,
            vec![redirect_uri1.clone(), redirect_uri2.clone()]
        );
        assert_eq!(res.grant_types, vec![CODE_GRANT_TYPE, REFRESH_GRANT_TYPE]);
        assert_eq!(res.response_types, vec![CODE_RESPONSE_TYPE]);
        assert!(res.client_secret_expires_at > 0);
        assert!(!res.client_secret.data().is_empty());
        let client_claims = decode_client_secret(&decoding_key, &res.client_secret).unwrap();
        assert_eq!(client_claims.exp, res.client_secret_expires_at);
        assert_eq!(client_claims.client_id, res.client_id);

        // Assert the client exists.
        let client = store.get_client(&res.client_id).await.unwrap().unwrap();
        assert_eq!(res.client_id, client.client_id);
        assert_eq!(res.client_name, client.client_name);
        assert_eq!(res.scope, client.scope);
        assert_eq!(
            res.token_endpoint_auth_method,
            client.token_endpoint_auth_method
        );
        assert_eq!(res.redirect_uris, client.redirect_uris);
        assert_eq!(res.grant_types, client.grant_types);
        assert_eq!(res.response_types, client.response_types);
        assert_eq!(
            res.client_secret_expires_at,
            client.client_secret_expires_at
        );
    }

    #[test_log::test(sqlx::test)]
    async fn test_oauth_register_minimal(pool: PgPool) {
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);
        let store = Store::new(Box::leak(Box::new(pool)));

        // Register the cliet
        let redirect_uri1: String = "http:://localhost/test/redirect1".to_string();
        let req = ClientRegistrationRequest {
            client_name: Some("Client 123".to_string()),
            scope: None,
            token_endpoint_auth_method: None,
            redirect_uris: vec![redirect_uri1.clone()],
            grant_types: vec![],
            response_types: vec![],
            other: Value::Null,
        };
        let Json(res) = oauth_register(Extension(key), Extension(store.clone()), Json(req))
            .await
            .unwrap();

        // Asserts over the response
        assert!(!res.client_id.is_empty());
        assert_eq!(res.client_name, "Client 123");
        assert_eq!(res.scope, READ_WRITE_SCOPE);
        assert_eq!(res.token_endpoint_auth_method, BASIC_AUTH_METHOD);
        assert_eq!(res.redirect_uris, vec![redirect_uri1.clone()]);
        assert_eq!(res.grant_types, vec![CODE_GRANT_TYPE, REFRESH_GRANT_TYPE]);
        assert_eq!(res.response_types, vec![CODE_RESPONSE_TYPE]);
        assert!(res.client_secret_expires_at > 0);
        assert!(!res.client_secret.data().is_empty());
        let client_claims = decode_client_secret(&decoding_key, &res.client_secret).unwrap();
        assert_eq!(client_claims.exp, res.client_secret_expires_at);
        assert_eq!(client_claims.client_id, res.client_id);

        // Assert the client exists.
        let client = store.get_client(&res.client_id).await.unwrap().unwrap();
        assert_eq!(res.client_id, client.client_id);
        assert_eq!(res.client_name, client.client_name);
        assert_eq!(res.scope, client.scope);
        assert_eq!(
            res.token_endpoint_auth_method,
            client.token_endpoint_auth_method
        );
        assert_eq!(res.redirect_uris, client.redirect_uris);
        assert_eq!(res.grant_types, client.grant_types);
        assert_eq!(res.response_types, client.response_types);
        assert_eq!(
            res.client_secret_expires_at,
            client.client_secret_expires_at
        );
    }

    #[test_log::test(sqlx::test)]
    async fn test_oauth_authorization_details(pool: PgPool) {
        let store = Store::new(Box::leak(Box::new(pool)));

        // Insert a client
        let client_id = "client-123".to_string();
        let client_name = "Client 123".to_string();
        let redirect_uri = "http:://localhost/test/redirect".to_string();
        store
            .insert_client(&ClientMetadata {
                client_id: client_id.clone(),
                client_name: client_name.to_string(),
                registered_at: 123,
                client_secret_expires_at: 9999999999999,
                scope: READ_WRITE_SCOPE.to_string(),
                token_endpoint_auth_method: BASIC_AUTH_METHOD.to_string(),
                redirect_uris: vec![redirect_uri.clone()],
                grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
                response_types: vec![CODE_RESPONSE_TYPE.to_string()],
            })
            .await
            .unwrap();

        let req = AuthorizationDetailsRequest {
            client_id: Some(client_id.clone()),
            redirect_uri: Some(redirect_uri.clone()),
            other: Value::Null,
        };

        // Success
        let Json(res) = oauth_authorization_details(Extension(store.clone()), Json(req))
            .await
            .unwrap();
        assert_eq!(res.client_name, client_name);

        // Invalid redirect URI
        let req = AuthorizationDetailsRequest {
            client_id: Some(client_id.clone()),
            redirect_uri: Some(format!("{redirect_uri}/bad")),
            other: Value::Null,
        };
        let res = oauth_authorization_details(Extension(store), Json(req)).await;
        let err = res.unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.error, "invalid_request");
        assert!(err.error_description.contains("uri doesn't match"));
    }

    #[test_log::test(sqlx::test)]
    async fn test_oauth_approve_full(pool: PgPool) {
        let user = User {
            email: "user@example.com".to_string(),
            name: "User".to_string(),
            picture: "pic".to_string(),
            exp: 9999999999,
        };
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);
        let store = Store::new(Box::leak(Box::new(pool)));

        // Insert a client
        let client_id = "client-123".to_string();
        let client_name = "Client 123".to_string();
        let redirect_uri = "http:://localhost/test/redirect".to_string();
        store
            .insert_client(&ClientMetadata {
                client_id: client_id.clone(),
                client_name: client_name.to_string(),
                registered_at: 123,
                client_secret_expires_at: 9999999999999,
                scope: READ_WRITE_SCOPE.to_string(),
                token_endpoint_auth_method: BASIC_AUTH_METHOD.to_string(),
                redirect_uris: vec![redirect_uri.clone()],
                grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
                response_types: vec![CODE_RESPONSE_TYPE.to_string()],
            })
            .await
            .unwrap();

        // Send the request
        let code_challenge = "5IcmHkDZ7xfw54Bs1U7ejZxKRZ5pxo0z_d5mCDWRlkc";
        let req = ApprovalRequest {
            client_id: Some(client_id.clone()),
            scope: Some(READ_WRITE_SCOPE.to_string()),
            response_type: Some(CODE_RESPONSE_TYPE.to_string()),
            code_challenge_method: Some("S256".to_string()),
            code_challenge: Some(code_challenge.to_string()),
            redirect_uri: Some(redirect_uri.clone()),
            resource: Some("Some/resource".to_string()),
            other: Value::Null,
        };
        let Json(res) = oauth_approve(
            Extension(user.clone()),
            Extension(store.clone()),
            Extension(key),
            Json(req),
        )
        .await
        .unwrap();
        assert!(!res.code.data().is_empty());

        // Assert over the resulting token.
        let token_claims: AuthTokenClaims = decode_auth_token(&decoding_key, &res.code).unwrap();
        let token = store
            .consume_auth_token(&token_claims.token_id)
            .await
            .unwrap();
        assert!(token.expires_at > 0);
        assert_eq!(token.client_id, client_id);
        assert_eq!(token.token_id, token_claims.token_id);
        assert_eq!(token.scope, READ_WRITE_SCOPE);
        assert_eq!(token.response_type, CODE_RESPONSE_TYPE);
        assert_eq!(token.redirect_uri, redirect_uri);
        assert_eq!(token.code_challenge_method, Some("S256".to_string()));
        assert_eq!(token.code_challenge, Some(code_challenge.to_string()));
        assert_eq!(token.user.email, user.email);
    }

    #[test_log::test(sqlx::test)]
    async fn test_oauth_approve_minimal(pool: PgPool) {
        let user = User {
            email: "user@example.com".to_string(),
            name: "User".to_string(),
            picture: "pic".to_string(),
            exp: 9999999999,
        };
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);
        let store = Store::new(Box::leak(Box::new(pool)));

        // Insert the client.
        let client_id = "client-123".to_string();
        let client_name = "Client 123".to_string();
        let redirect_uri = "http:://localhost/test/redirect".to_string();
        store
            .insert_client(&ClientMetadata {
                client_id: client_id.clone(),
                client_name: client_name.to_string(),
                registered_at: 123,
                client_secret_expires_at: 9999999999999,
                scope: READ_WRITE_SCOPE.to_string(),
                token_endpoint_auth_method: BASIC_AUTH_METHOD.to_string(),
                redirect_uris: vec![redirect_uri.clone()],
                grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
                response_types: vec![CODE_RESPONSE_TYPE.to_string()],
            })
            .await
            .unwrap();

        // Send the request
        let req = ApprovalRequest {
            client_id: Some(client_id.clone()),
            scope: None,
            response_type: None,
            code_challenge_method: None,
            code_challenge: None,
            redirect_uri: Some(redirect_uri.clone()),
            resource: None,
            other: Value::Null,
        };
        let Json(res) = oauth_approve(
            Extension(user.clone()),
            Extension(store.clone()),
            Extension(key),
            Json(req),
        )
        .await
        .unwrap();
        assert!(!res.code.data().is_empty());

        // Assert over the resulting token.
        let token_claims: AuthTokenClaims = decode_auth_token(&decoding_key, &res.code).unwrap();
        let token = store
            .consume_auth_token(&token_claims.token_id)
            .await
            .unwrap();
        assert!(token.expires_at > 0);
        assert_eq!(token.client_id, client_id);
        assert_eq!(token.token_id, token_claims.token_id);
        assert_eq!(token.scope, READ_WRITE_SCOPE);
        assert_eq!(token.response_type, CODE_RESPONSE_TYPE);
        assert_eq!(token.redirect_uri, redirect_uri);
        assert_eq!(token.code_challenge_method, None);
        assert_eq!(token.code_challenge, None);
        assert_eq!(token.user.email, user.email);
    }

    #[test_log::test(sqlx::test)]
    async fn test_oauth_token(pool: PgPool) {
        let user = User {
            email: "user@example.com".to_string(),
            name: "User".to_string(),
            picture: "pic".to_string(),
            exp: 9999999999,
        };
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);
        let store = Store::new(Box::leak(Box::new(pool)));
        let mut headers = HeaderMap::new();

        // Insert a client
        let client_id = "client-123".to_string();
        let code_verifier = "qkkIJx-brh0RjiBH3RJZeIiam66UuxHCZmsR7DxPuYB";
        let code_challenge = "5IcmHkDZ7xfw54Bs1U7ejZxKRZ5pxo0z_d5mCDWRlkc";
        let redirect_uri = "http:://localhost/test/redirect".to_string();
        let client_secret = encode_client_secret(
            &key,
            &ClientSecretClaims {
                iat: 123,
                exp: 99999999999999,
                iss: CLIENT_SECRET_ISS.to_string(),
                client_id: client_id.clone(),
            },
        )
        .unwrap();
        store
            .insert_client(&ClientMetadata {
                client_id: client_id.clone(),
                client_name: "Some client".to_string(),
                registered_at: 123,
                client_secret_expires_at: 9999999999999,
                scope: READ_WRITE_SCOPE.to_string(),
                token_endpoint_auth_method: BASIC_AUTH_METHOD.to_string(),
                redirect_uris: vec![redirect_uri.clone()],
                grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
                response_types: vec![CODE_RESPONSE_TYPE.to_string()],
            })
            .await
            .unwrap();

        // Store an auth token
        let token_id = "token-123".to_string();
        let code = encode_auth_token(
            &key,
            &AuthTokenClaims {
                iat: 123,
                exp: 99999999999999,
                iss: AUTH_TOKEN_ISS.to_string(),
                client_id: client_id.clone(),
                token_id: token_id.clone(),
            },
        )
        .unwrap();
        store
            .insert_auth_token(&AuthTokenMetadata {
                issued_at: 123,
                expires_at: 99999999999999,
                client_id: client_id.clone(),
                token_id: token_id.clone(),
                scope: READ_WRITE_SCOPE.to_string(),
                response_type: CODE_RESPONSE_TYPE.to_string(),
                redirect_uri: redirect_uri.clone(),
                code_challenge_method: Some("S256".to_string()),
                code_challenge: Some(code_challenge.to_string()),
                user: user.clone(),
            })
            .await
            .unwrap();

        // Send a token request
        let req = TokenRequest {
            grant_type: Some(CODE_GRANT_TYPE.to_string()),
            client_id: Some(client_id.clone()),
            client_secret: Some(client_secret.clone()),
            scope: Some(READ_WRITE_SCOPE.to_string()),
            resource: Some("resource/some".to_string()),
            redirect_uri: Some(redirect_uri.clone()),
            code: Some(code),
            code_verifier: Some(Secret::from(code_verifier)),
            refresh_token: None,
            other: Value::Null,
        };
        let res = oauth_token(
            Extension(store.clone()),
            Extension(decoding_key.clone()),
            Extension(key.clone()),
            headers.clone(),
            Form(req.clone()),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        let res: TokenResponse = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();

        // Asserts over the response
        assert_eq!(res.token_type, "Bearer");
        assert_eq!(res.scope, READ_WRITE_SCOPE);
        assert_eq!(res.expires_in, ACCESS_TOKEN_EXPIRY_SECS);
        assert!(!res.access_token.data().is_empty());
        assert!(!res.refresh_token.data().is_empty());
        let access_token = decode_access_token(&decoding_key, &res.access_token).unwrap();
        assert_eq!(access_token.user.email, user.email);
        assert_eq!(access_token.iss, ACCESS_TOKEN_ISS);
        assert_eq!(access_token.scope, READ_WRITE_SCOPE);
        assert_eq!(access_token.auth_token_claims.client_id, client_id);
        assert_eq!(access_token.auth_token_claims.token_id, token_id);
        let refresh_token = decode_refresh_token(&decoding_key, &res.refresh_token).unwrap();
        assert_eq!(access_token.user.email, user.email);
        assert_eq!(refresh_token.iss, REFRESH_TOKEN_ISS);
        assert_eq!(refresh_token.scope, READ_WRITE_SCOPE);
        assert_eq!(refresh_token.auth_token_claims.client_id, client_id);
        assert_eq!(refresh_token.auth_token_claims.token_id, token_id);
        // Ensure the token doesn't exist.
        assert!(store.consume_auth_token(&token_id).await.is_err());

        // Requests reusing a code are rejected.
        let err_res = oauth_token(
            Extension(store.clone()),
            Extension(decoding_key.clone()),
            Extension(key.clone()),
            headers.clone(),
            Form(req.clone()),
        )
        .await;
        assert_eq!(err_res.status(), StatusCode::BAD_REQUEST);
        let err: Value = serde_json::from_slice(
            &axum::body::to_bytes(err_res.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(err.get("error").unwrap().as_str().unwrap(), "invalid_grant");
        assert!(
            err.get("error_description")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("Auth code already used")
        );

        // Send a request using the refresh token issued in the last call
        let req = TokenRequest {
            grant_type: Some(REFRESH_GRANT_TYPE.to_string()),
            client_id: Some(client_id.clone()),
            client_secret: None,
            scope: None,
            resource: None,
            redirect_uri: None,
            code: None,
            code_verifier: None,
            refresh_token: Some(res.refresh_token),
            other: Value::Null,
        };
        headers.append(
            "Authorization",
            HeaderValue::from_str(&format!(
                "Basic {}",
                BASE64_STANDARD.encode(format!("{client_id}:{}", client_secret.data()))
            ))
            .unwrap(),
        );
        let res = oauth_token(
            Extension(store.clone()),
            Extension(decoding_key.clone()),
            Extension(key.clone()),
            headers.clone(),
            Form(req.clone()),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        let res: TokenResponse = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();

        // Assert over the response.
        assert_eq!(res.token_type, "Bearer");
        assert_eq!(res.scope, READ_WRITE_SCOPE);
        assert_eq!(res.expires_in, ACCESS_TOKEN_EXPIRY_SECS);
        assert!(!res.access_token.data().is_empty());
        assert!(!res.refresh_token.data().is_empty());
        let access_token = decode_access_token(&decoding_key, &res.access_token).unwrap();
        assert_eq!(access_token.user.email, user.email);
        assert_eq!(access_token.iss, ACCESS_TOKEN_ISS);
        assert_eq!(access_token.scope, READ_WRITE_SCOPE);
        assert_eq!(access_token.auth_token_claims.client_id, client_id);
        assert_eq!(access_token.auth_token_claims.token_id, token_id);
        let refresh_token = decode_refresh_token(&decoding_key, &res.refresh_token).unwrap();
        assert_eq!(access_token.user.email, user.email);
        assert_eq!(refresh_token.iss, REFRESH_TOKEN_ISS);
        assert_eq!(refresh_token.scope, READ_WRITE_SCOPE);
        assert_eq!(refresh_token.auth_token_claims.client_id, client_id);
        assert_eq!(refresh_token.auth_token_claims.token_id, token_id);
    }

    #[test_log::test(sqlx::test)]
    async fn mismatched_iss_fails_decoding() {
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);

        let token = encode_auth_token(
            &key,
            &AuthTokenClaims {
                iat: 123,
                exp: now().unwrap() + 9999,
                iss: ACCESS_TOKEN_ISS.to_string(),
                client_id: "client-1".to_string(),
                token_id: "token-1".to_string(),
            },
        )
        .unwrap();

        let err = decode_auth_token(&decoding_key, &token).unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.error, "invalid_grant");
    }

    #[test_log::test(sqlx::test)]
    async fn expired_token_fails_decoding() {
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);

        let token = encode_auth_token(
            &key,
            &AuthTokenClaims {
                iat: 123,
                exp: now().unwrap() - 500,
                iss: AUTH_TOKEN_ISS.to_string(),
                client_id: "client-1".to_string(),
                token_id: "token-1".to_string(),
            },
        )
        .unwrap();

        let err = decode_auth_token(&decoding_key, &token).unwrap_err();
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert_eq!(err.error, "invalid_grant");
    }

    #[test_log::test(sqlx::test)]
    async fn test_oauth_token_public_client(pool: PgPool) {
        let user = User {
            email: "user@example.com".to_string(),
            name: "User".to_string(),
            picture: "pic".to_string(),
            exp: 9999999999,
        };
        let key = EncodingKey::from_secret(&[1, 2]);
        let decoding_key = DecodingKey::from_secret(&[1, 2]);
        let store = Store::new(Box::leak(Box::new(pool)));
        let headers = HeaderMap::new();

        // Insert a client
        let client_id = "client-123".to_string();
        let code_verifier = "qkkIJx-brh0RjiBH3RJZeIiam66UuxHCZmsR7DxPuYB";
        let code_challenge = "5IcmHkDZ7xfw54Bs1U7ejZxKRZ5pxo0z_d5mCDWRlkc";
        let redirect_uri = "http:://localhost/test/redirect".to_string();
        store
            .insert_client(&ClientMetadata {
                client_id: client_id.clone(),
                client_name: "Some client".to_string(),
                registered_at: 123,
                client_secret_expires_at: 9999999999999,
                scope: READ_WRITE_SCOPE.to_string(),
                token_endpoint_auth_method: NONE_AUTH_METHOD.to_string(),
                redirect_uris: vec![redirect_uri.clone()],
                grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
                response_types: vec![CODE_RESPONSE_TYPE.to_string()],
            })
            .await
            .unwrap();

        // Store an auth token
        let token_id = "token-123".to_string();
        let code = encode_auth_token(
            &key,
            &AuthTokenClaims {
                iat: 123,
                exp: 99999999999999,
                iss: AUTH_TOKEN_ISS.to_string(),
                client_id: client_id.clone(),
                token_id: token_id.clone(),
            },
        )
        .unwrap();
        store
            .insert_auth_token(&AuthTokenMetadata {
                issued_at: 123,
                expires_at: 99999999999999,
                client_id: client_id.clone(),
                token_id: token_id.clone(),
                scope: READ_WRITE_SCOPE.to_string(),
                response_type: CODE_RESPONSE_TYPE.to_string(),
                redirect_uri: redirect_uri.clone(),
                code_challenge_method: Some("S256".to_string()),
                code_challenge: Some(code_challenge.to_string()),
                user: user.clone(),
            })
            .await
            .unwrap();

        // Send a token request
        let req = TokenRequest {
            grant_type: Some(CODE_GRANT_TYPE.to_string()),
            client_id: Some(client_id.clone()),
            client_secret: None,
            scope: Some(READ_WRITE_SCOPE.to_string()),
            resource: Some("resource/some".to_string()),
            redirect_uri: Some(redirect_uri.clone()),
            code: Some(code),
            code_verifier: Some(Secret::from(code_verifier)),
            refresh_token: None,
            other: Value::Null,
        };
        let res = oauth_token(
            Extension(store.clone()),
            Extension(decoding_key.clone()),
            Extension(key.clone()),
            headers.clone(),
            Form(req.clone()),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        let res: TokenResponse = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), 1024 * 1024)
                .await
                .unwrap(),
        )
        .unwrap();

        // Asserts over the response
        assert_eq!(res.token_type, "Bearer");
        assert_eq!(res.scope, READ_WRITE_SCOPE);
        assert_eq!(res.expires_in, ACCESS_TOKEN_EXPIRY_SECS);
        assert!(!res.access_token.data().is_empty());
        assert!(!res.refresh_token.data().is_empty());
        let access_token = decode_access_token(&decoding_key, &res.access_token).unwrap();
        assert_eq!(access_token.user.email, user.email);
        assert_eq!(access_token.iss, ACCESS_TOKEN_ISS);
        assert_eq!(access_token.scope, READ_WRITE_SCOPE);
        assert_eq!(access_token.auth_token_claims.client_id, client_id);
        assert_eq!(access_token.auth_token_claims.token_id, token_id);
        let refresh_token = decode_refresh_token(&decoding_key, &res.refresh_token).unwrap();
        assert_eq!(access_token.user.email, user.email);
        assert_eq!(refresh_token.iss, REFRESH_TOKEN_ISS);
        assert_eq!(refresh_token.scope, READ_WRITE_SCOPE);
        assert_eq!(refresh_token.auth_token_claims.client_id, client_id);
        assert_eq!(refresh_token.auth_token_claims.token_id, token_id);
        // Ensure the token doesn't exist.
        assert!(store.consume_auth_token(&token_id).await.is_err());
    }
}
