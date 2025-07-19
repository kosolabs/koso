use crate::{
    api::{
        ApiResult, IntoApiResult as _, IntoErrorResponse as _, bad_request_error,
        google::{self, User},
        unauthenticated_error,
    },
    settings::settings,
};
use anyhow::{Result, anyhow};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{self, Form},
    http::HeaderValue,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rmcp::transport::auth::{ClientRegistrationRequest, ClientRegistrationResponse};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
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
) -> ApiResult<Response<Body>> {
    // Parse the access token out of the Authorization header.
    let access_token = {
        let headers = request.headers();
        let Some(auth_header) = headers.get("Authorization") else {
            return Ok(unauthenticated("Authorization header is absent.")?);
        };

        let Ok(auth) = auth_header.to_str() else {
            return Ok(unauthenticated(&format!(
                "Could not convert auth header to string: {auth_header:?}"
            ))?);
        };
        let parts: Vec<&str> = auth.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Ok(unauthenticated(&format!(
                "Could not split bearer parts: {parts:?}"
            ))?);
        }
        parts[1]
    };
    // Decode the access token.
    let access_token_claims = decode_access_token(&decoding_key, access_token)?;

    let mut user = access_token_claims.access_token.user;
    user.email = user.email.to_lowercase();

    tracing::Span::current().record("email", user.email.clone());
    assert!(request.extensions_mut().insert(user).is_none());

    Ok(next.run(request).await)
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
async fn get_resource_server_metadata() -> ApiResult<Json<ResourceServerMetadata>> {
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
async fn get_authorization_server_metadata() -> ApiResult<Json<AuthorizationServerMetadata>> {
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

// Handle dynamic client registration
#[tracing::instrument(skip(key))]
async fn oauth_register(
    Extension(key): Extension<EncodingKey>,
    Json(req): Json<ClientRegistrationRequest>,
) -> ApiResult<Json<ClientRegistrationResponse>> {
    tracing::debug!("Registering client: {req:?}");

    if req.redirect_uris.is_empty() {
        return Err(bad_request_error(
            "invalid_request",
            "at least one redirect uri is required",
        ));
    }
    // TODO: validate more fields.
    // example: ClientRegistrationRequest { client_name: "MCP Inspector", redirect_uris: ["http://localhost:6274/oauth/callback"], grant_types: ["authorization_code", "refresh_token"], token_endpoint_auth_method: "none", response_types: ["code"] }

    let (client_secret, claims) = encode_client_secret(&key)?;
    let response = ClientRegistrationResponse {
        client_id: claims.client_id,
        client_secret: Some(client_secret),
        client_name: req.client_name,
        redirect_uris: req.redirect_uris,
        additional_fields: HashMap::new(),
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
    redirect_uri: Option<String>,
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
    scope: Option<String>,
    code_challenge_method: Option<String>,
    code_challenge: Option<String>,
    redirect_uri: Option<String>,
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
            access_token: format!("tp-token-{}", Uuid::new_v4()),
            scope: req.scope,
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
    // #[serde(default)]
    // redirect_uri: String,
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
    expires_in: u64,
}

// Handle token request from the MCP client
async fn oauth_token(
    Extension(decoding_key): Extension<DecodingKey>,
    Extension(encoding_key): Extension<EncodingKey>,
    Form(req): Form<TokenRequest>,
) -> ApiResult<Json<TokenResponse>> {
    let auth_token = if req.grant_type == "refresh_token" {
        tracing::info!("Handling refresh token request: {req:?}");
        let refresh_token_claims = decode_refresh_token(&decoding_key, &req.refresh_token)?;

        refresh_token_claims.refresh_token.auth_token
    } else {
        tracing::info!("Handling access token request: {req:?}");

        // Validate the request.
        if req.grant_type != "authorization_code" {
            return Err(
                anyhow!("Unsupported grant type: {}", req.grant_type).context_bad_request(
                    "unsupported_grant_type",
                    "only authorization_code is supported",
                ),
            );
        }
        let client_secret_claims = decode_client_secret(&decoding_key, &req.client_secret)?;
        if !req.client_id.is_empty() && client_secret_claims.client_id != req.client_id {
            return Err(bad_request_error("invalid_grant", "invalid client id"));
        }
        let auth_token_claims = decode_auth_token(&decoding_key, &req.code)?;
        if client_secret_claims.client_id != auth_token_claims.auth_token.client_id {
            return Err(bad_request_error("invalid_grant", "invalid client id"));
        }
        // if req.redirect_uri
        //     != auth_token_claims
        //         .auth_token
        //         .redirect_uri
        //         .as_deref()
        //         .unwrap_or_default()
        // {
        //     return Err(bad_request_error("invalid_grant", "invalid redirect uri"));
        // }

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
                    ));
                }
                let actual_challenge =
                    BASE64_URL_SAFE_NO_PAD.encode(Sha256::new().chain_update(verifier).finalize());
                if &actual_challenge != challenge {
                    return Err(bad_request_error(
                        "invalid_grant",
                        "Challenge does not match verifier",
                    ));
                }
            }
            (None, None, verifier) if verifier.is_empty() => {}
            _ => {
                return Err(bad_request_error(
                    "invalid_grant",
                    "Method, challenge and verifier must all be set or unset",
                ));
            }
        }

        auth_token_claims.auth_token
    };

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
    )?;
    let (refresh_token, refresh_claims) = encode_refresh_token(
        &encoding_key,
        RefreshToken {
            expires_in: 30 * 24 * 60 * 60, // 30 days
            client_id: auth_token.client_id.clone(),
            scope: auth_token.scope.clone(),
            user: auth_token.user.clone(),
            auth_token: auth_token.clone(),
        },
    )?;

    tracing::info!("Created access token: {access_claims:?}, refresh token: {refresh_claims:?}");

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: access_claims.access_token.expires_in,
    }))
}

const CLIENT_SECRET_ISS: &str = "koso-mcp-oauth-client";

#[derive(Serialize, Deserialize, Debug)]
struct ClientSecretClaims {
    exp: u64,
    iss: String,
    client_id: String,
    expires_in: u64,
}

fn encode_client_secret(key: &EncodingKey) -> ApiResult<(String, ClientSecretClaims)> {
    let client_id = format!("client-{}", Uuid::new_v4());

    let expires_in = 60 * 60;
    let timer = SystemTime::now() + Duration::from_secs(expires_in);
    let claims = ClientSecretClaims {
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: CLIENT_SECRET_ISS.to_string(),
        client_id: client_id.clone(),
        expires_in,
    };
    let client_secret = encode(&Header::default(), &claims, key)?;

    Ok((client_secret, claims))
}

fn decode_client_secret(key: &DecodingKey, client_secret: &str) -> ApiResult<ClientSecretClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(CLIENT_SECRET_ISS.to_string());
    validation.iss = Some(iss);
    let client_secret = decode::<ClientSecretClaims>(client_secret, key, &validation)
        .context_unauthorized("Invalid client secret token")?;

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
) -> ApiResult<(String, AuthTokenClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(auth_token.expires_in);
    let claims = AuthTokenClaims {
        auth_token,
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: AUTH_TOKEN_ISS.to_string(),
    };
    let token = encode(&Header::default(), &claims, key)?;

    Ok((token, claims))
}

fn decode_auth_token(key: &DecodingKey, auth_token: &str) -> ApiResult<AuthTokenClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(AUTH_TOKEN_ISS.to_string());
    validation.iss = Some(iss);
    let auth_token = decode::<AuthTokenClaims>(auth_token, key, &validation)
        .context_unauthorized("Invalid auth token")?;

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
    scope: Option<String>,
    auth_token: AuthToken,
}

fn encode_access_token(
    key: &EncodingKey,
    access_token: AccessToken,
) -> ApiResult<(String, AccessTokenClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(access_token.expires_in);
    let claims = AccessTokenClaims {
        access_token,
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: ACCESS_TOKEN_ISS.to_string(),
    };
    let token = encode(&Header::default(), &claims, key)?;

    Ok((token, claims))
}

fn decode_access_token(key: &DecodingKey, token: &str) -> ApiResult<AccessTokenClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(ACCESS_TOKEN_ISS.to_string());
    validation.iss = Some(iss);
    let token: jsonwebtoken::TokenData<AccessTokenClaims> =
        decode::<AccessTokenClaims>(token, key, &validation)
            .context_unauthorized("Invalid access token")?;

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
    scope: Option<String>,
    auth_token: AuthToken,
}

fn encode_refresh_token(
    key: &EncodingKey,
    refresh_token: RefreshToken,
) -> ApiResult<(String, RefreshTokenClaims)> {
    let timer = SystemTime::now() + Duration::from_secs(refresh_token.expires_in);
    let claims = RefreshTokenClaims {
        refresh_token,
        exp: timer.duration_since(UNIX_EPOCH)?.as_secs(),
        iss: REFRESH_TOKEN_ISS.to_string(),
    };
    let token = encode(&Header::default(), &claims, key)?;

    Ok((token, claims))
}

fn decode_refresh_token(key: &DecodingKey, token: &str) -> ApiResult<RefreshTokenClaims> {
    let mut validation = Validation::default();
    let mut iss = HashSet::new();
    iss.insert(REFRESH_TOKEN_ISS.to_string());
    validation.iss = Some(iss);
    let token: jsonwebtoken::TokenData<RefreshTokenClaims> =
        decode::<RefreshTokenClaims>(token, key, &validation)
            .context_unauthorized("Invalid refresh token")?;

    Ok(token.claims)
}

fn unauthenticated(msg: &str) -> Result<Response<Body>> {
    let mut res = unauthenticated_error(msg).into_response();
    res.headers_mut().insert(
        "WWW-Authenticate",
        HeaderValue::from_str(&format!(
            "Bearer resource_metadata={}/.well-known/oauth-protected-resource",
            settings().host
        ))?,
    );
    Ok(res)
}

// TODO Error bodies: https://www.rfc-editor.org/rfc/rfc6749#section-5.2
