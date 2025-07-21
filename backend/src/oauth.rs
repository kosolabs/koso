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
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::{NoneAsEmptyString, serde_as};
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
        .nest(
            "/.well-known/oauth-authorization-server",
            Router::new()
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
                    "/approve",
                    post(oauth_approve)
                        .options(oauth_approve)
                        .layer(middleware::from_fn(google::authenticate)),
                )
                .fallback(api::handler_404),
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

    let mut user = access_token_claims.user;
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

const READ_WRITE_SCOPE: &str = "read_write";
const CODE_RESPONSE_TYPE: &str = "code";
const CODE_GRANT_TYPE: &str = "authorization_code";
const REFRESH_GRANT_TYPE: &str = "refresh_token";

/// oauth2 resource server metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ResourceServerMetadata {
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

/// https://datatracker.ietf.org/doc/html/rfc8414
#[tracing::instrument()]
async fn get_authorization_server_metadata() -> OauthResult<Json<AuthorizationServerMetadata>> {
    let host = &settings().host;
    let metadata = AuthorizationServerMetadata {
        authorization_endpoint: format!("{host}/connections/mcp/oauth/authorize"),
        token_endpoint: format!("{host}/oauth/token"),
        registration_endpoint: format!("{host}/oauth/register"),
        scopes_supported: vec![READ_WRITE_SCOPE.to_string()],
        issuer: host.clone(),
        response_types_supported: vec![CODE_RESPONSE_TYPE.to_string()],
        code_challenge_methods_supported: vec!["S256".to_string()],
    };
    tracing::debug!("Metadata: {:?}", metadata);

    Ok(Json(metadata))
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientRegistrationRequest {
    #[serde_as(as = "NoneAsEmptyString")]
    client_name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    scope: Option<String>,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientRegistrationResponse {
    client_id: String,
    client_name: String,
    scope: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
    client_secret_expires_at: u64,
    client_secret: String,
}

// Handle dynamic client registration
// https://datatracker.ietf.org/doc/html/rfc7591#section-3.1
#[tracing::instrument(skip(key))]
async fn oauth_register(
    Extension(key): Extension<EncodingKey>,
    Json(req): Json<ClientRegistrationRequest>,
) -> OauthResult<Json<ClientRegistrationResponse>> {
    tracing::debug!("Registering client");

    // Validate the request
    let redirect_uris = req.redirect_uris;
    if redirect_uris.is_empty() {
        return Err(bad_request_error(
            "invalid_redirect_uri",
            "At least one redirect uri is required",
        )
        .into());
    }
    if redirect_uris.len() > 5 {
        return Err(bad_request_error("invalid_redirect_uri", "Too many redirect uris").into());
    }
    if redirect_uris.iter().any(|s| s.is_empty()) {
        return Err(
            bad_request_error("invalid_redirect_uri", "Redirect uri cannot be empty").into(),
        );
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

    // Generate the client secret.
    let client_id = format!("client-{}", Uuid::new_v4());
    let claims = ClientSecretClaims {
        exp: expires_at(CLIENT_SECRET_EXPIRY_SECS)?,
        iss: CLIENT_SECRET_ISS.to_string(),
        client_id: format!("client-{}", Uuid::new_v4()),
        client_name: req.client_name.unwrap_or_else(|| client_id.clone()),
        expires_in: CLIENT_SECRET_EXPIRY_SECS,
        scope,
        redirect_uris,
        grant_types: vec![CODE_GRANT_TYPE.to_string(), REFRESH_GRANT_TYPE.to_string()],
        response_types: vec![CODE_RESPONSE_TYPE.to_string()],
    };
    let client_secret = encode_client_secret(&key, &claims)?;

    // Create the response.
    let response = ClientRegistrationResponse {
        client_id: claims.client_id,
        client_name: claims.client_name,
        scope: claims.scope,
        redirect_uris: claims.redirect_uris,
        grant_types: claims.grant_types,
        response_types: claims.response_types,
        client_secret_expires_at: claims.exp,
        client_secret,
    };
    tracing::debug!("Registered client: {response:?}");

    Ok(Json(response))
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct ApprovalRequest {
    #[serde_as(as = "NoneAsEmptyString")]
    client_id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    scope: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    response_type: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    code_challenge_method: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    code_challenge: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    redirect_uri: Option<String>,
    #[allow(dead_code)]
    #[serde_as(as = "NoneAsEmptyString")]
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
#[tracing::instrument(skip(user, encoding_key))]
async fn oauth_approve(
    Extension(user): Extension<User>,
    Extension(encoding_key): Extension<EncodingKey>,
    Json(req): Json<ApprovalRequest>,
) -> ApiResult<Json<ApprovalResponse>> {
    tracing::info!("Approving authorization");

    // Validate the request.
    // TODO: validate redirect_uri against the registered client.
    // TODO: validate the token hasn't already been used.
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
            "invalid_client_metadata",
            "Only read_write scope is supported",
        ));
    }
    let response_type = req
        .response_type
        .unwrap_or_else(|| CODE_RESPONSE_TYPE.to_string());
    if response_type != CODE_RESPONSE_TYPE {
        return Err(bad_request_error(
            "invalid_client_metadata",
            "Only the 'code' response_type is supported",
        ));
    }
    let (code_challenge_method, code_challenge) = (req.code_challenge_method, req.code_challenge);
    match (&code_challenge_method, &code_challenge) {
        (Some(method), Some(challenge)) if !method.is_empty() && !challenge.is_empty() => {
            if method != "S256" {
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

    // Encode the auth token
    let auth_token_claims = AuthTokenClaims {
        exp: expires_at(AUTH_TOKEN_EXPIRY_SECS)?,
        iss: AUTH_TOKEN_ISS.to_string(),
        client_id,
        expires_in: AUTH_TOKEN_EXPIRY_SECS,
        scope,
        response_type,
        redirect_uri,
        code_challenge,
        code_challenge_method,
        access_token: format!("token-{}", Uuid::new_v4()),
        user,
    };
    let auth_token = encode_auth_token(&encoding_key, &auth_token_claims)?;

    tracing::info!("Approved authorization, created auth token: {auth_token_claims:?}");

    Ok(Json(ApprovalResponse { code: auth_token }))
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct TokenRequest {
    /// refresh_token or authorization_code
    #[serde_as(as = "NoneAsEmptyString")]
    grant_type: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    client_id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    client_secret: Option<String>,

    // authorization_code fields
    #[serde_as(as = "NoneAsEmptyString")]
    code: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    code_verifier: Option<String>,

    // refresh_token_fields
    #[serde_as(as = "NoneAsEmptyString")]
    refresh_token: Option<String>,

    #[allow(dead_code)]
    #[serde_as(as = "NoneAsEmptyString")]
    redirect_uri: Option<String>,
    #[allow(dead_code)]
    #[serde_as(as = "NoneAsEmptyString")]
    resource: Option<String>,

    #[allow(dead_code)]
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    token_type: String,
    expires_in: u64,
    scope: String,
    access_token: String,
    refresh_token: String,
}

/// Handle token request from the MCP client
/// https://datatracker.ietf.org/doc/html/rfc6749#section-3.2
#[tracing::instrument(skip(decoding_key, encoding_key))]
async fn oauth_token(
    Extension(decoding_key): Extension<DecodingKey>,
    Extension(encoding_key): Extension<EncodingKey>,
    Form(req): Form<TokenRequest>,
) -> OauthResult<Json<TokenResponse>> {
    let (client_id, scope, user, auth_token_claims) = match req.grant_type.as_deref() {
        Some(REFRESH_GRANT_TYPE) => {
            tracing::info!("Handling refresh token request");

            // Decode the refresh token and grab the auth token.
            let Some(refresh_token) = req.refresh_token else {
                return Err(bad_request_error(
                    "invalid_request",
                    "refresh_token required for refresh_token grant type",
                )
                .into());
            };
            let refresh_token = decode_refresh_token(&decoding_key, &refresh_token)
                .context_bad_request("invalid_grant", "Invalid refresh token")?;

            (
                refresh_token.client_id,
                refresh_token.scope,
                refresh_token.user,
                refresh_token.auth_token_claims,
            )
        }
        Some(CODE_GRANT_TYPE) => {
            tracing::info!("Handling access token request");

            // Decode the auth token.
            let Some(code) = req.code else {
                return Err(bad_request_error(
                    "invalid_request",
                    "code required for authorization_code grant type",
                )
                .into());
            };
            let auth_token_claims = decode_auth_token(&decoding_key, &code)
                .context_bad_request("invalid_grant", "Invalid auth token")?;

            // PKCE: Verify the challenge against the verifier.
            match (
                &auth_token_claims.code_challenge_method,
                &auth_token_claims.code_challenge,
                req.code_verifier,
            ) {
                (Some(method), Some(challenge), Some(verifier)) => {
                    if method != "S256" {
                        return Err(bad_request_error(
                            "unsupported_grant_type",
                            "Only S256 is supported",
                        )
                        .into());
                    }
                    let actual_challenge = BASE64_URL_SAFE_NO_PAD
                        .encode(Sha256::new().chain_update(verifier).finalize());
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

            (
                auth_token_claims.client_id.clone(),
                auth_token_claims.scope.clone(),
                auth_token_claims.user.clone(),
                auth_token_claims,
            )
        }
        Some(_) => {
            return Err(bad_request_error(
                "unsupported_grant_type",
                "only authorization_code is supported",
            )
            .into());
        }
        None => {
            return Err(bad_request_error("invalid_request", "grant_type required").into());
        }
    };

    let Some(req_client_id) = req.client_id else {
        return Err(bad_request_error("invalid_request", "Client id required").into());
    };
    if req_client_id != client_id {
        return Err(bad_request_error("invalid_grant", "Invalid client id").into());
    }
    let Some(client_secret) = req.client_secret else {
        return Err(bad_request_error(
            "invalid_request",
            "client_secret required for authorization_code grant type",
        )
        .into());
    };
    let client_secret_claims = decode_client_secret(&decoding_key, &client_secret)
        .context_bad_request("invalid_grant", "Invalid client secret")?;
    if client_secret_claims.client_id != client_id {
        return Err(
            bad_request_error("invalid_grant", "Auth token issued to another client").into(),
        );
    }

    // Encode the access token and refresh token.
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

const CLIENT_SECRET_ISS: &str = "koso-mcp-oauth-client";
const CLIENT_SECRET_EXPIRY_SECS: u64 = 30 * 24 * 60 * 60;
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientSecretClaims {
    exp: u64,
    iss: String,
    client_id: String,
    client_name: String,
    expires_in: u64,
    scope: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    response_types: Vec<String>,
}

const AUTH_TOKEN_ISS: &str = "koso-mcp-oauth-auth";
const AUTH_TOKEN_EXPIRY_SECS: u64 = 10 * 60;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthTokenClaims {
    exp: u64,
    iss: String,
    client_id: String,
    expires_in: u64,
    scope: String,
    response_type: String,
    redirect_uri: String,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    access_token: String,
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
const REFRESH_TOKEN_EXPIRY_SECS: u64 = 30 * 24 * 60 * 60;
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

fn expires_at(expires_in: u64) -> ApiResult<u64> {
    let timer = SystemTime::now() + Duration::from_secs(expires_in);
    Ok(timer.duration_since(UNIX_EPOCH)?.as_secs())
}

fn decode_token<T: DeserializeOwned>(key: &DecodingKey, token: &str, issuer: &str) -> Result<T> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(issuer.to_string());
    validation.iss = Some(iss);
    validation.required_spec_claims.insert("iss".to_string());
    let token: jsonwebtoken::TokenData<T> =
        decode::<T>(token, key, &validation).context("Invalid token")?;

    Ok(token.claims)
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
                .unwrap_or("server_error")
                .to_string(),
            error_description: detail
                .map(|d| d.msg.as_str())
                .unwrap_or("Internal error, something went wrong")
                .to_string(),
        }
    }
}
