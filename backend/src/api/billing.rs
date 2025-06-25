use std::collections::HashMap;

use crate::api::billing::model::{
    CreateCheckoutSessionRequest, CreateCheckoutSessionResponse, CreatePortalSessionRequest,
    CreatePortalSessionResponse,
};
use crate::api::{ApiResult, google::User};
use crate::api::{bad_request_error, google, unauthorized_error};
use crate::notifiers::UserNotificationConfig;
use crate::secrets::{self, Secret};
use crate::settings::settings;
use anyhow::{Context, Result, anyhow};
use axum::body::{Body, Bytes};
use axum::http::request::Parts;
use axum::middleware;
use axum::{Extension, Json, Router, routing::post};
use chrono::{DateTime, TimeDelta, Utc};
use hmac::{Hmac, Mac};
use reqwest::IntoUrl;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::Sha256;
use sqlx::{FromRow, PgPool};

#[derive(Clone)]
struct WebhookSecret(Secret<Vec<u8>>);

#[derive(Clone)]
struct StripeClient {
    client: reqwest::Client,
    secret_key: Secret<String>,
}

pub(crate) fn router() -> Result<Router> {
    let secret_key = secrets::read_secret("stripe/secret_key")?;
    let webhook_secret = WebhookSecret(secrets::read_secret("stripe/webhook_secret")?);
    let client = StripeClient {
        client: reqwest::Client::new(),
        secret_key,
    };

    Ok(Router::new()
        .route(
            "/stripe/create-checkout-session",
            post(create_checkout_session),
        )
        .route("/stripe/create-portal-session", post(create_portal_session))
        .layer((middleware::from_fn(google::authenticate),))
        // The webhook endpoint is invoked by Stripe and not users. Don't authenticate using Google.
        .route("/stripe/webhook", post(handle_webhook))
        .layer((Extension(client),))
        .layer((Extension(webhook_secret),)))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Profile {
    notification_configs: Vec<UserNotificationConfig>,
    plugin_connections: PluginConnections,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
#[serde(rename_all = "camelCase")]
struct PluginConnections {
    github_user_id: Option<String>,
}

#[derive(Serialize, Debug)]
struct CreateCheckoutSession<'a> {
    mode: &'a str,
    success_url: &'a str,
    cancel_url: &'a str,
    client_reference_id: &'a str,
    customer_email: Option<&'a str>,
    customer: Option<&'a str>,
    line_items: Vec<CreateCheckoutSessionLineItems<'a>>,
    subscription_data: CreateCheckoutSessionSubscriptionData,
    metadata: KosoMetadata,
}

#[derive(Serialize, Debug)]
struct CreateCheckoutSessionLineItems<'a> {
    quantity: u64,
    price: &'a str,
    adjustable_quantity: CreateCheckoutSessionAdjustableQuantity,
}

#[derive(Serialize, Debug)]
struct CreateCheckoutSessionAdjustableQuantity {
    enabled: bool,
    maximum: i16,
    minimum: i16,
}

#[derive(Serialize, Debug)]
struct CreateCheckoutSessionSubscriptionData {
    metadata: KosoMetadata,
}

#[derive(Deserialize, Debug)]
struct CheckoutSessionResponse {
    id: String,
    url: String,
}

#[tracing::instrument(skip(user, pool, client))]
async fn create_checkout_session(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(client): Extension<StripeClient>,
    Json(request): Json<CreateCheckoutSessionRequest>,
) -> ApiResult<Json<CreateCheckoutSessionResponse>> {
    // TODO: Validate request
    // TODO: verify the URL's hostname is ours.
    let session: CheckoutSessionResponse = {
        let customer_id = get_stripe_customer_id(&user, pool).await?;

        let mut params = CreateCheckoutSession {
            mode: "subscription",
            success_url: &request.success_url,
            cancel_url: &request.cancel_url,
            client_reference_id: &user.email,
            customer_email: None,
            customer: customer_id.as_deref(),
            line_items: vec![CreateCheckoutSessionLineItems {
                quantity: 5,
                price: &settings().stripe.price_id,
                adjustable_quantity: CreateCheckoutSessionAdjustableQuantity {
                    enabled: true,
                    maximum: 200,
                    minimum: 1,
                },
            }],
            subscription_data: CreateCheckoutSessionSubscriptionData {
                metadata: KosoMetadata {
                    email: Some(user.email.clone()),
                },
            },
            metadata: KosoMetadata {
                email: Some(user.email.clone()),
            },
        };
        if customer_id.is_none() {
            params.customer_email = Some(&user.email);
        }

        client
            .post("https://api.stripe.com/v1/checkout/sessions", &params)
            .await
            .context("Failed to create checkout session")?
    };
    tracing::info!("Created Stripe CheckoutSession. ID={}", session.id);

    Ok(Json(CreateCheckoutSessionResponse {
        redirect_url: session.url,
    }))
}

#[derive(Serialize, Debug)]
struct CreatePortalSession<'a> {
    customer: &'a str,
    return_url: &'a str,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct PortalSessionResponse {
    id: String,
    url: String,
}

#[tracing::instrument(skip(user, pool, client))]
async fn create_portal_session(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(client): Extension<StripeClient>,
    Json(request): Json<CreatePortalSessionRequest>,
) -> ApiResult<Json<CreatePortalSessionResponse>> {
    // TODO: Validate request
    // TODO: verify the URL's hostname is ours.
    let session: PortalSessionResponse = {
        let Some(customer_id) = get_stripe_customer_id(&user, pool).await? else {
            return Err(bad_request_error(
                "CUSTOMER_ID_UNSET",
                "Customer ID is not set",
            ));
        };
        let params: CreatePortalSession = CreatePortalSession {
            customer: &customer_id,
            return_url: &request.return_url,
        };
        client
            .post("https://api.stripe.com/v1/billing_portal/sessions", &params)
            .await
            .context("Failed to create portal session")?
    };
    tracing::info!("Created Stripe BillingPortalSession. ID={}", session.id);

    Ok(Json(CreatePortalSessionResponse {
        redirect_url: session.url,
    }))
}

async fn get_stripe_customer_id(user: &User, pool: &PgPool) -> Result<Option<String>> {
    match sqlx::query_as(
        "
        SELECT stripe_customer_id
        FROM subscriptions
        WHERE email = $1;
        ",
    )
    .bind(&user.email)
    .fetch_optional(pool)
    .await
    .context("Failed to fetch stripe customer id")?
    {
        Some((Some(customer_id),)) => Ok(Some(customer_id)),
        None | Some((None,)) => Ok(None),
    }
}

impl StripeClient {
    async fn post<U: IntoUrl, T: Serialize, R: DeserializeOwned>(
        &self,
        url: U,
        params: T,
    ) -> Result<R> {
        let res = self
            .client
            .post(url)
            .header("Stripe-Version", "2025-05-28.basil")
            .header("User-Agent", "koso-backend")
            .header("content-type", "application/x-www-form-urlencoded")
            .bearer_auth(&self.secret_key.data)
            .body(serde_qs::to_string(&params).context("Failed to serialize params")?)
            .send()
            .await
            .context("Failed to send post")?;
        if !res.status().is_success() {
            return Err(anyhow!(
                "Post failed with status {}: {:?}>>{:?}",
                res.status(),
                format!("{:?}", res.headers()),
                res.text().await,
            ));
        }
        res.json().await.context("Failed to deserialize response")
    }

    async fn get<U: IntoUrl, R: DeserializeOwned>(&self, url: U) -> Result<R> {
        let res = self
            .client
            .get(url)
            .header("Stripe-Version", "2025-05-28.basil")
            .header("User-Agent", "koso-backend")
            .header("content-type", "application/x-www-form-urlencoded")
            .bearer_auth(&self.secret_key.data)
            .send()
            .await
            .context("Failed to send post")?;
        if !res.status().is_success() {
            return Err(anyhow!(
                "Get failed with status {}: {:?}>>{:?}",
                res.status(),
                format!("{:?}", res.headers()),
                res.text().await,
            ));
        }
        res.json().await.context("Failed to deserialize response")
    }

    async fn get_subscription(&self, id: &str) -> Result<Subscription> {
        self.get(format!("https://api.stripe.com/v1/subscriptions/{}", id))
            .await
            .context("Failed to get subscription")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Event<'a> {
    id: String,

    #[serde(rename = "type")]
    type_: String,

    #[serde(borrow)]
    data: &'a RawValue,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CheckoutSessionObject {
    object: CheckoutSession,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CheckoutSession {
    id: String,
    client_reference_id: String,
    customer: String,
    subscription: String,
    metadata: KosoMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionObject {
    object: Subscription,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Subscription {
    id: String,
    customer: String,
    quantity: i32,
    status: String,
    metadata: KosoMetadata,
    items: SubscriptionItems,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionItems {
    data: Vec<SubscriptionItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionItem {
    id: String,
    current_period_end: i64,
    quantity: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceObject {
    object: Invoice,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Invoice {
    id: String,
    customer: String,
    status: String,
    parent: InvoiceParent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceParent {
    subscription_details: InvoiceParentSubscription,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceParentSubscription {
    subscription: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct KosoMetadata {
    email: Option<String>,
}

/// Maximum size of request body in bytes.
const BODY_LIMIT: usize = 10 * 1024 * 1024;

#[tracing::instrument(
    skip(webhook_secret, pool, client, parts, body),
    fields(stripe_event, stripe_event_id)
)]
async fn handle_webhook(
    Extension(webhook_secret): Extension<WebhookSecret>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(client): Extension<StripeClient>,
    parts: Parts,
    body: Body,
) -> ApiResult<()> {
    let body: Bytes = axum::body::to_bytes(body, BODY_LIMIT)
        .await
        .map_err(|_| bad_request_error("INVALID_BODY", "Invalid body"))?;
    if let Some(signature) = parts.headers.get("stripe-signature") {
        validate_signature(signature.to_str()?, &body, &webhook_secret)?;
    } else if !settings().stripe.enable_unathenticated_webhook {
        return Err(bad_request_error(
            "MISSING_HEADER",
            "Missing stripe-signature header",
        ));
    };

    let event: Event = serde_json::from_slice(&body)
        .map_err(|e| bad_request_error("INVALID_REQUEST", &format!("Invalid request: {e}")))?;
    tracing::Span::current().record("stripe_event", event.type_.to_string());
    tracing::Span::current().record("stripe_event_id", event.id.to_string());

    // https://docs.stripe.com/billing/subscriptions/build-subscriptions?platform=web&ui=stripe-hosted&lang=node
    // https://docs.stripe.com/billing/subscriptions/webhooks#active-subscriptions
    match event.type_.as_str() {
        // Payment is successful and the subscription is created.
        "checkout.session.completed" => {
            let session: CheckoutSessionObject = serde_json::from_str(event.data.get())?;
            let session = session.object;
            tracing::info!("Processing completed session event: {session:?}");

            let subscription = client.get_subscription(&session.subscription).await?;
            apply_subscription(pool, &subscription).await?;
        }

        // TODO: Do we need this in addition to the invoice events below?
        "customer.subscription.created"
        | "customer.subscription.updated"
        | "customer.subscription.deleted"
        | "customer.subscription.paused"
        | "customer.subscription.resumed" => {
            let subscription: SubscriptionObject = serde_json::from_str(event.data.get())?;
            let subscription = subscription.object;
            tracing::info!("Processing subscription event: {subscription:?}");

            let subscription = client.get_subscription(&subscription.id).await?;
            apply_subscription(pool, &subscription).await?;
        }

        // Continue to provision the subscription as payments continue to be made.
        "invoice.paid" => {
            let invoice: InvoiceObject = serde_json::from_str(event.data.get())?;
            let invoice = invoice.object;
            tracing::info!("Processing invoice event: {invoice:?}");

            let subscription = client
                .get_subscription(&invoice.parent.subscription_details.subscription)
                .await?;
            apply_subscription(pool, &subscription).await?;
        }

        _ => tracing::trace!("Unknown event encountered in webhook: {:?}", event.type_),
    }

    Ok(())
}

async fn apply_subscription(pool: &PgPool, subscription: &Subscription) -> Result<()> {
    tracing::info!("Applying subscription {subscription:?}");

    let item = subscription
        .items
        .data
        .first()
        .context("Unexpectedly got no subscription items")?;
    // https://docs.stripe.com/billing/subscriptions/webhooks#state-changes
    let end_time = if subscription.status == "canceled" || subscription.status == "unpaid" {
        Utc::now()
            .checked_sub_signed(TimeDelta::minutes(5))
            .context("could not sub delta")?
    } else {
        DateTime::from_timestamp(item.current_period_end, 0)
            .context("could not convert to timesetamp")?
    };
    let seats = item.quantity;
    let email = subscription
        .metadata
        .email
        .as_deref()
        .context("Subscription metadata email absent")?;

    let mut txn = pool.begin().await?;
    // First, insert (or update) the subscription
    sqlx::query(
        "
        INSERT INTO subscriptions (email, stripe_customer_id, seats, end_time, member_emails)
        VALUES ($1, $2, $3, $4, ARRAY[$1])
        ON CONFLICT (email)
        DO UPDATE
        SET seats = EXCLUDED.seats, end_time = EXCLUDED.end_time
        WHERE subscriptions.seats!=EXCLUDED.seats OR subscriptions.end_time!=EXCLUDED.end_time",
    )
    .bind(email)
    .bind(&subscription.customer)
    .bind(seats)
    .bind(end_time)
    .execute(&mut *txn)
    .await
    .context("Failed to upsert subscription")?;

    // For each member of the target subscription, set the latest end time of all their subscriptions.
    sqlx::query(
        "
        UPDATE users u1
        SET subscription_end_time=subquery.end_time
        FROM (
            SELECT email, MAX(end_time) AS end_time
            FROM (
                SELECT UNNEST(member_emails) AS email
                FROM subscriptions
                WHERE email=$1
            ) JOIN(
                SELECT UNNEST(member_emails) AS email, end_time
                FROM subscriptions
            ) USING(email)
            GROUP BY email
        ) subquery
        WHERE u1.email=subquery.email AND (u1.subscription_end_time IS NULL OR u1.subscription_end_time!=subquery.end_time)",
    )
    .bind(email)
    .execute(&mut *txn)
    .await
    .context("Failed to update member end times")?;

    txn.commit().await?;

    Ok(())
}

/// Validate the authenticity of the event.
/// See https://docs.stripe.com/webhooks?verify=verify-manually
fn validate_signature(
    signature_header: &str,
    payload: &[u8],
    secret: &WebhookSecret,
) -> ApiResult<()> {
    let (timestamp, signature) = match parse_signature(signature_header) {
        Ok(v) => v,
        Err(err) => {
            return Err(unauthorized_error(&format!(
                "Invalid signature header {}: {err}",
                signature_header
            )));
        }
    };

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret.0.data)?;
    mac.update(timestamp.to_string().as_bytes());
    mac.update(".".as_bytes());
    mac.update(payload);

    if let Err(err) = mac.verify_slice(&signature) {
        tracing::warn!("Received webhook event with invalid signature: {err:?}");
        return Err(unauthorized_error(&format!(
            "Invalid signature: {}",
            hex::encode(signature)
        )));
    }

    // Get current timestamp to compare to signature timestamp
    if (Utc::now().timestamp() - timestamp).abs() > 300 {
        return Err(unauthorized_error(&format!(
            "stale event at time {timestamp}"
        )));
    }

    Ok(())
}

fn parse_signature(signature_header: &str) -> Result<(i64, Vec<u8>)> {
    let headers: HashMap<&str, &str> = signature_header
        .split(',')
        .map(|header| {
            let mut key_and_value = header.split('=');
            let key = key_and_value.next();
            let value = key_and_value.next();
            (key, value)
        })
        .filter_map(|(key, value)| match (key, value) {
            (Some(key), Some(value)) => Some((key, value)),
            _ => None,
        })
        .collect();
    let timestamp = headers
        .get("t")
        .context("missing 't' key")?
        .parse::<i64>()?;
    let signature = hex::decode(headers.get("v1").context("missing 'v1' key")?)?;

    Ok((timestamp, signature))
}

pub(crate) mod model {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreateCheckoutSessionRequest {
        pub(crate) success_url: String,
        pub(crate) cancel_url: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreateCheckoutSessionResponse {
        pub(crate) redirect_url: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreatePortalSessionRequest {
        pub(crate) return_url: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreatePortalSessionResponse {
        pub(crate) redirect_url: String,
    }
}
