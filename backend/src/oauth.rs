use crate::{
    api::{
        ApiResult, ErrorResponse, IntoApiResult, bad_request_error,
        google::{self, User},
    },
    settings::settings,
};
use anyhow::{Context as _, Result, anyhow};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{self, Form},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tower_http::cors::{self, CorsLayer};
use uuid::Uuid;

pub(crate) fn router() -> Result<Router> {
    let cors_layer = CorsLayer::new()
        .allow_origin(cors::Any)
        .allow_methods(cors::Any)
        .allow_headers(cors::Any);

    Ok(Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(get_authorization_server_metadata).options(get_authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(get_resource_server_metadata).options(get_resource_server_metadata),
        )
        .route(
            "/oauth/register",
            post(oauth_register).options(oauth_register),
        )
        .route("/oauth/token", post(oauth_token).options(oauth_token))
        .layer(cors_layer)
        .route(
            "/oauth/approve",
            post(oauth_approve)
                .options(oauth_approve)
                .layer(middleware::from_fn(google::authenticate)),
        ))
}

/// Middleware function that authenticates requests to the MCP server.
#[tracing::instrument(skip(request, next, decoding_key), fields(email))]
pub(crate) async fn authenticate(
    Extension(decoding_key): Extension<DecodingKey>,
    mut request: extract::Request,
    next: Next,
) -> OauthResult<Response<Body>> {
    let access_token_claims: AccessTokenClaims = _authenticate(&decoding_key, &mut request)
        .context_status(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "Unauthenticated client",
        )?;

    let mut user = access_token_claims.access_token.user;
    user.email = user.email.to_lowercase();

    tracing::Span::current().record("email", user.email.clone());
    assert!(request.extensions_mut().insert(user).is_none());

    Ok(next.run(request).await)
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

/// oauth2 resource server metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ResourceServerMetadata {
    resource: String,
    authorization_servers: Vec<String>,
    bearer_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

#[tracing::instrument()]
async fn get_resource_server_metadata() -> OauthResult<Json<ResourceServerMetadata>> {
    let host = &settings().host;
    let metadata = ResourceServerMetadata {
        resource: format!("{host}/api/mcp/sse"),
        authorization_servers: vec!["http:localhost:3000/api/foo".to_string()],
        bearer_methods_supported: vec!["header".to_string()],
        scopes_supported: vec!["profile".to_string(), "email".to_string()],
    };
    tracing::debug!("Metadata: {:?}", metadata);

    Ok(Json(metadata))
}

/// oauth2 authorization server metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuthorizationServerMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
    registration_endpoint: String,
    issuer: String,
    scopes_supported: Vec<String>,
    response_types_supported: Vec<String>,
    code_challenge_methods_supported: Vec<String>,
}

#[tracing::instrument()]
async fn get_authorization_server_metadata() -> OauthResult<Json<AuthorizationServerMetadata>> {
    let host = &settings().host;
    let metadata = AuthorizationServerMetadata {
        authorization_endpoint: format!("{host}/connections/mcp/oauth/authorize"),
        token_endpoint: format!("{host}/oauth/token"),
        scopes_supported: vec!["profile".to_string(), "email".to_string()],
        registration_endpoint: format!("{host}/oauth/register"),
        issuer: host.clone(),
        response_types_supported: vec!["code".to_string()],
        code_challenge_methods_supported: vec!["S256".to_string()],
    };
    tracing::debug!("Metadata: {:?}", metadata);

    Ok(Json(metadata))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientRegistrationRequest {
    client_name: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    // token_endpoint_auth_method: String,
    response_types: Vec<String>,
    scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientRegistrationResponse {
    client_id: String,
    client_secret: String,
    client_secret_expires_at: u64,
    client_name: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    scope: String,
}

// Handle dynamic client registration
// https://datatracker.ietf.org/doc/html/rfc7591#section-3.1
#[tracing::instrument(skip(key))]
async fn oauth_register(
    Extension(key): Extension<EncodingKey>,
    Json(req): Json<ClientRegistrationRequest>,
) -> OauthResult<Json<ClientRegistrationResponse>> {
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
    let response_types = {
        if !req.response_types.is_empty() {
            req.response_types
        } else {
            vec!["code".to_string()]
        }
    };
    if !response_types.contains(&"code".to_string()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only the 'code' response_type is supported",
        )
        .into());
    }
    let grant_types = {
        if !req.grant_types.is_empty() {
            req.grant_types
        } else {
            vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ]
        }
    };
    if !grant_types.contains(&"authorization_code".to_string()) {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "grant_types must contain 'authorization_code'",
        )
        .into());
    }
    let scope = validate_scope(req.scope)?;

    // Generate the client secret.
    let client_id = format!("client-{}", Uuid::new_v4());
    let client_name = if req.client_name.is_empty() {
        client_id.clone()
    } else {
        req.client_name
    };
    let (client_secret, claims) = encode_client_secret(
        &key,
        Client {
            client_id,
            client_name,
            expires_in: 30 * 24 * 60 * 60,
            redirect_uris,
            grant_types,
            response_types,
            scope,
        },
    )
    .context_internal("Internal error")?;

    // Create the response.
    let response = ClientRegistrationResponse {
        client_id: claims.client.client_id,
        client_secret,
        client_secret_expires_at: claims.exp,
        client_name: claims.client.client_name,
        redirect_uris: claims.client.redirect_uris,
        response_types: claims.client.response_types,
        grant_types: claims.client.grant_types,
        scope: claims.client.scope,
    };
    tracing::debug!("Registered client: {response:?}");

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct ApprovalRequest {
    client_id: String,
    scope: Option<String>,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    redirect_uri: String,
}

#[derive(Debug, Serialize)]
struct ApprovalResponse {
    code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthToken {
    client_id: String,
    access_token: String,
    expires_in: u64,
    scope: String,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    redirect_uri: String,
    user: User,
}

#[tracing::instrument(skip(user, encoding_key))]
async fn oauth_approve(
    Extension(user): Extension<User>,
    Extension(encoding_key): Extension<EncodingKey>,
    Json(req): Json<ApprovalRequest>,
) -> ApiResult<Json<ApprovalResponse>> {
    tracing::info!("Approving authorization: {req:?}");

    // Validate the request.
    if req.client_id.is_empty() {
        return Err(bad_request_error("invalid_request", "Client id required"));
    }
    if req.redirect_uri.is_empty() {
        return Err(bad_request_error(
            "invalid_request",
            "Redirect uri required",
        ));
    }
    // TODO: validate redirect_uri against the registered client.
    // TODO: validate the token hasn't already been used.
    let scope = validate_scope(req.scope)?;

    match (&req.code_challenge_method, &req.code_challenge) {
        (Some(method), Some(challenge)) => {
            if method.is_empty() || challenge.is_empty() {
                return Err(bad_request_error(
                    "invalid_request",
                    "Method and code must both be non-empty",
                ));
            }
        }
        (None, None) => {}
        _ => {
            return Err(bad_request_error(
                "invalid_request",
                "Method and code must both be set",
            ));
        }
    }

    // Encode the auth token
    let (auth_token, auth_token_claims) = encode_auth_token(
        &encoding_key,
        AuthToken {
            client_id: req.client_id,
            expires_in: 10 * 60,
            access_token: format!("token-{}", Uuid::new_v4()),
            scope,
            code_challenge: req.code_challenge,
            code_challenge_method: req.code_challenge_method,
            redirect_uri: req.redirect_uri,
            user,
        },
    )?;

    tracing::info!("Approved authorization, created auth token: {auth_token_claims:?}");

    Ok(Json(ApprovalResponse { code: auth_token }))
}

#[derive(Debug, Deserialize)]
struct TokenRequest {
    /// refresh_token or authorization_code
    grant_type: String,

    // authorization_code fields
    #[serde(default)]
    code: String,
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    client_secret: String,
    #[serde(default)]
    code_verifier: String,

    // refresh_token_fields
    #[serde(default)]
    refresh_token: String,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    scope: String,
    expires_in: u64,
}

// Handle token request from the MCP client
async fn oauth_token(
    Extension(decoding_key): Extension<DecodingKey>,
    Extension(encoding_key): Extension<EncodingKey>,
    Form(req): Form<TokenRequest>,
) -> OauthResult<Json<TokenResponse>> {
    let auth_token = if req.grant_type == "refresh_token" {
        tracing::info!("Handling refresh token request: {req:?}");

        // Validate the request.
        if req.refresh_token.is_empty() {
            return Err(bad_request_error(
                "invalid_request",
                "refresh_token required for refresh_token grant type",
            )
            .into());
        }

        // Decode the refresh token and grab the auth token.
        let refresh_token_claims = decode_refresh_token(&decoding_key, &req.refresh_token)
            .context_bad_request("invalid_grant", "Invalid refresh token")?;
        refresh_token_claims.refresh_token.auth_token
    } else {
        tracing::info!("Handling access token request: {req:?}");

        // Validate the request.
        if req.grant_type != "authorization_code" {
            return Err(bad_request_error(
                "unsupported_grant_type",
                "only authorization_code is supported",
            )
            .into());
        }
        if req.code.is_empty() {
            return Err(bad_request_error(
                "invalid_request",
                "code required for authorization_code grant type",
            )
            .into());
        }

        // Decode the auth token.
        let auth_token_claims = decode_auth_token(&decoding_key, &req.code)
            .context_bad_request("invalid_grant", "Invalid auth token")?;

        // PKCE: Verify the challenge against the verifier.
        match (
            &auth_token_claims.auth_token.code_challenge_method,
            &auth_token_claims.auth_token.code_challenge,
            req.code_verifier,
        ) {
            (Some(method), Some(challenge), verifier) if !verifier.is_empty() => {
                if method != "S256" {
                    return Err(bad_request_error(
                        "unsupported_grant_type",
                        "Only S256 is supported",
                    )
                    .into());
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
            (None, None, verifier) if verifier.is_empty() => {}
            _ => {
                return Err(bad_request_error(
                    "invalid_grant",
                    "Method, challenge and verifier must all be set or unset",
                )
                .into());
            }
        }

        auth_token_claims.auth_token
    };

    if req.client_secret.is_empty() {
        return Err(bad_request_error(
            "invalid_request",
            "client_secret required for authorization_code grant type",
        )
        .into());
    }
    let client_secret_claims = decode_client_secret(&decoding_key, &req.client_secret)
        .context_bad_request("invalid_grant", "Invalid client secret")?;
    if client_secret_claims.client.client_id != auth_token.client_id {
        return Err(bad_request_error("invalid_grant", "Invalid token or secret").into());
    }
    if auth_token.client_id != req.client_id {
        return Err(bad_request_error("invalid_grant", "Invalid client id").into());
    }

    // Encode the access token and refresh token.
    let (access_token, access_claims) = encode_access_token(
        &encoding_key,
        AccessToken {
            expires_in: 60 * 2, // TODO: Bump this
            client_id: auth_token.client_id.clone(),
            scope: auth_token.scope.clone(),
            user: auth_token.user.clone(),
            auth_token: auth_token.clone(),
        },
    )
    .context_internal("Internal error")?;
    let (refresh_token, refresh_claims) = encode_refresh_token(
        &encoding_key,
        RefreshToken {
            expires_in: 30 * 24 * 60 * 60, // 30 days
            client_id: auth_token.client_id.clone(),
            scope: auth_token.scope.clone(),
            user: auth_token.user.clone(),
            auth_token: auth_token.clone(),
        },
    )
    .context_internal("Internal error")?;

    tracing::info!("Created access token: {access_claims:?}, refresh token: {refresh_claims:?}");

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: access_claims.access_token.expires_in,
        scope: access_claims.access_token.scope,
    }))
}

const CLIENT_SECRET_ISS: &str = "koso-mcp-oauth-client";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Client {
    client_id: String,
    client_name: String,
    scope: String,
    expires_in: u64,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientSecretClaims {
    exp: u64,
    iss: String,
    client: Client,
}

fn encode_client_secret(key: &EncodingKey, client: Client) -> Result<(String, ClientSecretClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(client.expires_in);
    let claims = ClientSecretClaims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: CLIENT_SECRET_ISS.to_string(),
        client,
    };
    let client_secret = encode(&Header::default(), &claims, key)?;

    Ok((client_secret, claims))
}

fn decode_client_secret(key: &DecodingKey, client_secret: &str) -> Result<ClientSecretClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(CLIENT_SECRET_ISS.to_string());
    validation.iss = Some(iss);
    let client_secret = decode::<ClientSecretClaims>(client_secret, key, &validation)
        .context("Invalid client secret token")?;

    Ok(client_secret.claims)
}

const AUTH_TOKEN_ISS: &str = "koso-mcp-oauth-auth";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthTokenClaims {
    auth_token: AuthToken,
    exp: u64,
    iss: String,
}

fn encode_auth_token(
    key: &EncodingKey,
    auth_token: AuthToken,
) -> Result<(String, AuthTokenClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(auth_token.expires_in);
    let claims = AuthTokenClaims {
        auth_token,
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: AUTH_TOKEN_ISS.to_string(),
    };
    let token = encode(&Header::default(), &claims, key)?;

    Ok((token, claims))
}

fn decode_auth_token(key: &DecodingKey, auth_token: &str) -> Result<AuthTokenClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(AUTH_TOKEN_ISS.to_string());
    validation.iss = Some(iss);
    let auth_token =
        decode::<AuthTokenClaims>(auth_token, key, &validation).context("Invalid auth token")?;

    Ok(auth_token.claims)
}

const ACCESS_TOKEN_ISS: &str = "koso-mcp-oauth-access";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    exp: u64,
    iss: String,
    access_token: AccessToken,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AccessToken {
    client_id: String,
    user: User,
    expires_in: u64,
    scope: String,
    auth_token: AuthToken,
}

fn encode_access_token(
    key: &EncodingKey,
    access_token: AccessToken,
) -> Result<(String, AccessTokenClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(access_token.expires_in);
    let claims = AccessTokenClaims {
        access_token,
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: ACCESS_TOKEN_ISS.to_string(),
    };
    let token = encode(&Header::default(), &claims, key)?;

    Ok((token, claims))
}

fn decode_access_token(key: &DecodingKey, token: &str) -> Result<AccessTokenClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(ACCESS_TOKEN_ISS.to_string());
    validation.iss = Some(iss);
    let token: jsonwebtoken::TokenData<AccessTokenClaims> =
        decode::<AccessTokenClaims>(token, key, &validation).context("Invalid access token")?;

    Ok(token.claims)
}

const REFRESH_TOKEN_ISS: &str = "koso-mcp-oauth-refresh";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    exp: u64,
    iss: String,
    refresh_token: RefreshToken,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RefreshToken {
    client_id: String,
    user: User,
    expires_in: u64,
    scope: String,
    auth_token: AuthToken,
}

fn encode_refresh_token(
    key: &EncodingKey,
    refresh_token: RefreshToken,
) -> Result<(String, RefreshTokenClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(refresh_token.expires_in);
    let claims = RefreshTokenClaims {
        refresh_token,
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: REFRESH_TOKEN_ISS.to_string(),
    };
    let token = encode(&Header::default(), &claims, key)?;

    Ok((token, claims))
}

fn decode_refresh_token(key: &DecodingKey, token: &str) -> Result<RefreshTokenClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(REFRESH_TOKEN_ISS.to_string());
    validation.iss = Some(iss);
    let token: jsonwebtoken::TokenData<RefreshTokenClaims> =
        decode::<RefreshTokenClaims>(token, key, &validation).context("Invalid refresh token")?;

    Ok(token.claims)
}

fn validate_scope(scope: Option<String>) -> ApiResult<String> {
    let scope = scope
        .and_then(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .unwrap_or("email profile".to_string());
    for scope in scope.split_ascii_whitespace() {
        if scope != "profile" && scope != "email" {
            return Err(bad_request_error(
                "invalid_client_metadata",
                "Only profile or email scope is supported",
            ));
        }
    }
    Ok(scope)
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
impl IntoResponse for OauthErrorResponse {
    fn into_response(self) -> Response {
        let body = axum::Json(OauthErrorResponseBody {
            error: self.error,
            error_description: self.error_description,
        });

        let mut res = (self.status, body).into_response();
        match HeaderValue::from_str(&format!(
            "Bearer resource_metadata={}/.well-known/oauth-protected-resource",
            settings().host
        )) {
            Ok(header_value) => {
                res.headers_mut().insert("WWW-Authenticate", header_value);
            }
            Err(err) => {
                tracing::error!("Failed to crate authenticate header value: ${err:#}");
            }
        };
        res
    }
}

impl From<ErrorResponse> for OauthErrorResponse {
    fn from(err: ErrorResponse) -> Self {
        let detail = err.details.first();
        OauthErrorResponse {
            status: err.status,
            error: detail
                .map(|d| d.reason)
                .unwrap_or("internal_error")
                .to_string(),
            error_description: detail
                .map(|d| d.msg.as_str())
                .unwrap_or("Internal error, something went wrong")
                .to_string(),
        }
    }
}

// TODO Error bodies: https://www.rfc-editor.org/rfc/rfc6749#section-5.2
