use crate::{
    api::{
        ApiResult, bad_request_error,
        billing::{
            model::{
                CreateCheckoutSessionRequest, CreateCheckoutSessionResponse,
                CreatePortalSessionRequest, CreatePortalSessionResponse, Subscription,
                SubscriptionStatus, UpdateSubscriptionRequest, UpdateSubscriptionResponse,
            },
            stripe::{KosoMetadata, StripeClient},
            webhook::WebhookSecret,
        },
        google::User,
        not_found_error,
    },
    secrets::{self},
    settings::settings,
};
use anyhow::{Context, Result};
use axum::routing::put;
use axum::{Extension, Json, Router, routing::post};
use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool, Postgres};
use std::collections::HashMap;

pub(super) fn router() -> Result<Router> {
    let secret_key = secrets::read_secret("stripe/secret_key")?;
    let webhook_secret = WebhookSecret(secrets::read_secret("stripe/webhook_secret")?);
    let client = StripeClient {
        client: reqwest::Client::new(),
        secret_key,
    };

    Ok(Router::new()
        .route(
            "/stripe/create-checkout-session",
            post(handle_create_checkout_session),
        )
        .route(
            "/stripe/create-portal-session",
            post(handle_create_portal_session),
        )
        .route("/subscriptions", put(handle_update_subscription))
        .route("/stripe/webhook", post(webhook::handle_webhook))
        .layer((Extension(client),))
        .layer((Extension(webhook_secret),)))
}

#[tracing::instrument(skip(user, pool, client))]
async fn handle_create_checkout_session(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&PgPool>,
    Extension(client): Extension<StripeClient>,
    Json(request): Json<CreateCheckoutSessionRequest>,
) -> ApiResult<Json<CreateCheckoutSessionResponse>> {
    // TODO: Validate request
    // TODO: verify the URL's hostname is ours.
    let session = {
        let customer_id = get_stripe_customer_id(&user, pool).await?;

        let mut params = stripe::CreateCheckoutSession {
            mode: "subscription",
            success_url: &request.success_url,
            cancel_url: &request.cancel_url,
            client_reference_id: &user.email,
            customer_email: None,
            customer: customer_id.as_deref(),
            line_items: vec![stripe::CreateCheckoutSessionLineItems {
                quantity: 5,
                price: &settings().stripe.price_id,
                adjustable_quantity: stripe::CreateCheckoutSessionAdjustableQuantity {
                    enabled: true,
                    maximum: 200,
                    minimum: 1,
                },
            }],
            subscription_data: stripe::CreateCheckoutSessionSubscriptionData {
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

        client.create_checkout_session(&params).await?
    };
    tracing::info!("Created Stripe CheckoutSession. ID={}", session.id);

    Ok(Json(CreateCheckoutSessionResponse {
        redirect_url: session.url,
    }))
}

#[tracing::instrument(skip(user, pool, client))]
async fn handle_create_portal_session(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&PgPool>,
    Extension(client): Extension<StripeClient>,
    Json(request): Json<CreatePortalSessionRequest>,
) -> ApiResult<Json<CreatePortalSessionResponse>> {
    // TODO: Validate request
    // TODO: verify the URL's hostname is ours.
    let session = {
        let Some(customer_id) = get_stripe_customer_id(&user, pool).await? else {
            return Err(bad_request_error(
                "CUSTOMER_ID_UNSET",
                "Customer ID is not set",
            ));
        };

        let params = stripe::CreatePortalSession {
            customer: &customer_id,
            return_url: &request.return_url,
        };
        client.create_portal_session(&params).await?
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

#[tracing::instrument(skip(user, pool))]
async fn handle_update_subscription(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&PgPool>,
    Json(request): Json<UpdateSubscriptionRequest>,
) -> ApiResult<Json<UpdateSubscriptionResponse>> {
    let mut desired_members = request
        .members
        .into_iter()
        .map(|e| (e.to_lowercase(), true))
        .collect::<HashMap<String, bool>>()
        .into_keys()
        .collect::<Vec<String>>();
    desired_members.sort();
    if !desired_members.contains(&user.email) {
        return Err(bad_request_error(
            "MISSING_SELF",
            "Members must include owner",
        ));
    }

    let mut txn = pool.begin().await?;
    let Some((seats, existing_members, _)) = get_subscription_details(&user, &mut txn).await?
    else {
        return Err(not_found_error("NOT_FOUND", "Subscription not found"));
    };

    let added_members = desired_members
        .iter()
        .filter(|m| !existing_members.contains(m))
        .collect::<Vec<_>>();
    let removed_members = existing_members
        .iter()
        .filter(|m| !desired_members.contains(m))
        .collect::<Vec<_>>();
    // Nothing has changed, return immediately.
    if added_members.is_empty() && removed_members.is_empty() {
        return Ok(Json(UpdateSubscriptionResponse {}));
    }

    // Don't allow more members to be added than there are seats available.
    if !added_members.is_empty() && desired_members.len() > usize::try_from(seats)? {
        return Err(bad_request_error(
            "TOO_MANY_MEMBERS",
            &format!(
                "Tried to put {} members in {seats} seats",
                desired_members.len()
            ),
        ));
    }

    set_members(&user, &desired_members, &mut txn).await?;
    for member in added_members.into_iter().chain(removed_members) {
        update_user_subscription_end_time(member, &mut *txn).await?;
    }

    txn.commit().await?;

    Ok(Json(UpdateSubscriptionResponse {}))
}

async fn get_subscription_details(
    user: &User,
    txn: &mut PgConnection,
) -> Result<Option<(i32, Vec<String>, DateTime<Utc>)>> {
    sqlx::query_as(
        "
        SELECT seats, member_emails, end_time
        FROM subscriptions
        WHERE email=$1",
    )
    .bind(&user.email)
    .fetch_optional(txn)
    .await
    .context("Failed to fetch seats")
}

async fn set_members(user: &User, members: &Vec<String>, txn: &mut PgConnection) -> Result<()> {
    sqlx::query(
        "
        UPDATE subscriptions
        SET member_emails=$2
        WHERE email=$1",
    )
    .bind(&user.email)
    .bind(members)
    .execute(txn)
    .await
    .context("Failed to upsert subscription")?;
    Ok(())
}

/// Updates the given users subscription_end_time to match their current subscriptions.
pub(crate) async fn update_user_subscription_end_time<
    'a,
    E: sqlx::Executor<'a, Database = Postgres>,
>(
    email: &str,
    txn: E,
) -> Result<()> {
    sqlx::query(
        "
                UPDATE users u1
                SET subscription_end_time=(
                    SELECT MAX(end_time) AS end_time
                    FROM subscriptions
                    WHERE u1.email=ANY(member_emails)
                )
                WHERE u1.email=$1",
    )
    .bind(email)
    .execute(txn)
    .await
    .context("Failed to update member end times")?;

    Ok(())
}

pub(crate) async fn fetch_owned_subscription(
    email: &str,
    pool: &PgPool,
) -> Result<Option<Subscription>> {
    Ok(sqlx::query_as(
        "
        SELECT seats, end_time, member_emails
        FROM subscriptions
        WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .context("Failed to query user subscriptions")?
    .map(
        |(seats, end_time, mut member_emails): (i32, DateTime<Utc>, Vec<String>)| {
            // Sort for consistent ordering.
            member_emails.sort();
            Subscription {
                seats,
                end_time,
                member_emails,
                status: if end_time.timestamp() <= chrono::Utc::now().timestamp() {
                    SubscriptionStatus::Expired
                } else {
                    SubscriptionStatus::Active
                },
            }
        },
    ))
}

mod stripe {
    use crate::secrets::Secret;
    use anyhow::{Context, Result, anyhow};
    use reqwest::{IntoUrl, Response};
    use serde::{Deserialize, Serialize, de::DeserializeOwned};

    #[derive(Clone)]
    pub(super) struct StripeClient {
        pub(super) client: reqwest::Client,
        pub(super) secret_key: Secret<String>,
    }

    #[derive(Serialize, Debug)]
    pub(super) struct CreateCheckoutSession<'a> {
        pub(super) mode: &'a str,
        pub(super) success_url: &'a str,
        pub(super) cancel_url: &'a str,
        pub(super) client_reference_id: &'a str,
        pub(super) customer_email: Option<&'a str>,
        pub(super) customer: Option<&'a str>,
        pub(super) line_items: Vec<CreateCheckoutSessionLineItems<'a>>,
        pub(super) subscription_data: CreateCheckoutSessionSubscriptionData,
        pub(super) metadata: KosoMetadata,
    }

    #[derive(Serialize, Debug)]
    pub(super) struct CreateCheckoutSessionLineItems<'a> {
        pub(super) quantity: u64,
        pub(super) price: &'a str,
        pub(super) adjustable_quantity: CreateCheckoutSessionAdjustableQuantity,
    }

    #[derive(Serialize, Debug)]
    pub(super) struct CreateCheckoutSessionAdjustableQuantity {
        pub(super) enabled: bool,
        pub(super) maximum: i16,
        pub(super) minimum: i16,
    }

    #[derive(Serialize, Debug)]
    pub(super) struct CreateCheckoutSessionSubscriptionData {
        pub(super) metadata: KosoMetadata,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct CheckoutSessionResponse {
        pub(super) id: String,
        pub(super) url: String,
    }

    #[derive(Serialize, Debug)]
    pub(super) struct CreatePortalSession<'a> {
        pub(super) customer: &'a str,
        pub(super) return_url: &'a str,
    }

    #[derive(Clone, Deserialize, Serialize, Debug)]
    pub(super) struct PortalSessionResponse {
        pub(super) id: String,
        pub(super) url: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub(in crate::api::billing) struct Subscription {
        pub(crate) id: String,
        pub(crate) customer: String,
        pub(crate) quantity: i32,
        pub(crate) status: String,
        pub(crate) metadata: KosoMetadata,
        pub(crate) items: SubscriptionItems,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub(in crate::api::billing) struct SubscriptionItems {
        pub(crate) data: Vec<SubscriptionItem>,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub(in crate::api::billing) struct SubscriptionItem {
        pub(crate) id: String,
        pub(crate) current_period_end: i64,
        pub(crate) quantity: i32,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub(in crate::api::billing) struct KosoMetadata {
        pub(crate) email: Option<String>,
    }

    const API_VERSION: &str = "2025-05-28.basil";
    const USER_AGENT: &str = "koso-backend";

    impl StripeClient {
        async fn post<U: IntoUrl, T: Serialize, R: DeserializeOwned>(
            &self,
            url: U,
            params: T,
        ) -> Result<R> {
            let res = self
                .client
                .post(url)
                .header("Stripe-Version", API_VERSION)
                .header("User-Agent", USER_AGENT)
                .header("content-type", "application/x-www-form-urlencoded")
                .bearer_auth(&self.secret_key.data)
                .body(serde_qs::to_string(&params).context("Failed to serialize params")?)
                .send()
                .await
                .context("Failed to send post")?;
            Self::parse_response(res).await
        }

        async fn get<U: IntoUrl, R: DeserializeOwned>(&self, url: U) -> Result<R> {
            let res = self
                .client
                .get(url)
                .header("Stripe-Version", API_VERSION)
                .header("User-Agent", USER_AGENT)
                .header("content-type", "application/x-www-form-urlencoded")
                .bearer_auth(&self.secret_key.data)
                .send()
                .await
                .context("Failed to send post")?;
            Self::parse_response(res).await
        }

        async fn parse_response<R: DeserializeOwned>(res: Response) -> Result<R> {
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

        pub(super) async fn get_subscription(&self, id: &str) -> Result<Subscription> {
            self.get(format!("https://api.stripe.com/v1/subscriptions/{id}"))
                .await
                .context("Failed to get subscription")
        }

        pub(super) async fn create_checkout_session(
            &self,
            params: &CreateCheckoutSession<'_>,
        ) -> Result<CheckoutSessionResponse> {
            self.post("https://api.stripe.com/v1/checkout/sessions", params)
                .await
                .context("Failed to create checkout session")
        }

        pub(super) async fn create_portal_session(
            &self,
            params: &CreatePortalSession<'_>,
        ) -> Result<PortalSessionResponse> {
            self.post("https://api.stripe.com/v1/billing_portal/sessions", params)
                .await
                .context("Failed to create portal session")
        }
    }
}

mod webhook {
    use crate::{
        api::{
            ApiResult, bad_request_error,
            billing::stripe::{KosoMetadata, StripeClient, Subscription},
            unauthorized_error,
        },
        secrets::Secret,
        settings::settings,
    };
    use anyhow::{Context, Result};
    use axum::{
        Extension,
        body::{Body, Bytes},
        http::HeaderMap,
    };
    use chrono::{DateTime, TimeDelta, Utc};
    use hmac::{Hmac, Mac};
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue;
    use sha2::Sha256;
    use sqlx::PgPool;
    use std::collections::HashMap;

    #[derive(Clone)]
    pub(super) struct WebhookSecret(pub(super) Secret<Vec<u8>>);

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct Event<'a> {
        id: String,

        #[serde(rename = "type")]
        type_: String,

        #[serde(borrow)]
        data: &'a RawValue,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct CheckoutSessionObject {
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
    struct SubscriptionObject {
        object: Subscription,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct InvoiceObject {
        object: Invoice,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct Invoice {
        id: String,
        customer: String,
        status: String,
        parent: InvoiceParent,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct InvoiceParent {
        subscription_details: InvoiceParentSubscription,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct InvoiceParentSubscription {
        subscription: String,
    }

    /// Maximum size of request body in bytes.
    const BODY_LIMIT: usize = 10 * 1024 * 1024;

    #[tracing::instrument(
        skip(webhook_secret, pool, client, headers, body),
        fields(stripe_event, stripe_event_id)
    )]
    pub(super) async fn handle_webhook(
        Extension(webhook_secret): Extension<WebhookSecret>,
        Extension(pool): Extension<&PgPool>,
        Extension(client): Extension<StripeClient>,
        headers: HeaderMap,
        body: Body,
    ) -> ApiResult<()> {
        let body: Bytes = axum::body::to_bytes(body, BODY_LIMIT)
            .await
            .map_err(|_| bad_request_error("INVALID_BODY", "Invalid body"))?;

        // First, authenticate the event by validating the signature.
        if let Some(signature) = headers.get("stripe-signature") {
            validate_signature(signature.to_str()?, &body, &webhook_secret)?;
        } else if !settings().stripe.enable_unauthenticated_webhook {
            return Err(bad_request_error(
                "MISSING_HEADER",
                "Missing stripe-signature header",
            ));
        };

        // Parse the event.
        let event: Event = serde_json::from_slice(&body)
            .map_err(|e| bad_request_error("INVALID_REQUEST", &format!("Invalid request: {e}")))?;
        tracing::Span::current().record("stripe_event", event.type_.to_string());
        tracing::Span::current().record("stripe_event_id", event.id.to_string());

        // Process the event.
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

            _ => tracing::debug!("Unknown event encountered in webhook: {:?}", event.type_),
        }

        Ok(())
    }

    async fn apply_subscription(pool: &PgPool, subscription: &Subscription) -> Result<()> {
        tracing::info!("Applying subscription {subscription:?}");

        // Grab details from the subscription.
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
                .context("could not convert to timestamp")?
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
            SET stripe_customer_id = EXCLUDED.stripe_customer_id, seats = EXCLUDED.seats, end_time = EXCLUDED.end_time
            WHERE
                subscriptions.stripe_customer_id!=EXCLUDED.stripe_customer_id
                OR subscriptions.seats!=EXCLUDED.seats
                OR subscriptions.end_time!=EXCLUDED.end_time",
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
                    "Invalid signature header {signature_header}: {err}"
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
}

pub(crate) mod model {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreateCheckoutSessionRequest {
        pub(crate) success_url: String,
        pub(crate) cancel_url: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreateCheckoutSessionResponse {
        pub(crate) redirect_url: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreatePortalSessionRequest {
        pub(crate) return_url: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CreatePortalSessionResponse {
        pub(crate) redirect_url: String,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct UpdateSubscriptionRequest {
        pub(crate) members: Vec<String>,
    }
    #[derive(Serialize, Debug)]
    pub(crate) struct UpdateSubscriptionResponse {}

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Subscription {
        pub(crate) status: SubscriptionStatus,
        pub(crate) seats: i32,
        pub(crate) end_time: DateTime<Utc>,
        pub(crate) member_emails: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) enum SubscriptionStatus {
        None,
        Active,
        Expired,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::billing::webhook::handle_webhook, secrets::Secret};
    use axum::{body::Body, http::HeaderMap};
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;

    #[test_log::test(sqlx::test)]
    async fn create_checkout_session(pool: PgPool) {
        let user = User {
            email: "stripe-test@test.koso.app".to_string(),
            name: "IntegTesting DoNotDelete".to_string(),
            picture: "".to_string(),
            exp: 5,
        };
        let client = StripeClient {
            client: reqwest::Client::new(),
            secret_key: secrets::read_secret("stripe/secret_key").unwrap(),
        };
        let request = CreateCheckoutSessionRequest {
            success_url: "http://localhost/success".to_string(),
            cancel_url: "http://localhost/success".to_string(),
        };
        let Json(res) = handle_create_checkout_session(
            Extension(user),
            Extension(&pool),
            Extension(client),
            Json(request),
        )
        .await
        .unwrap();
        assert!(!res.redirect_url.is_empty());
    }

    #[test_log::test(sqlx::test)]
    async fn create_portal_session(pool: PgPool) {
        sqlx::query(
            "
            INSERT INTO users (email, name, picture, subscription_end_time, github_user_id)
            VALUES ('stripe-test@test.koso.app', 'IntegTesting DoNotDelete', '', null, null)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "
            INSERT INTO subscriptions (email, stripe_customer_id, seats, end_time, member_emails)
            VALUES ('stripe-test@test.koso.app', 'cus_SZSSBiHc9f8eQ6', 5, TIMESTAMP '2100-01-20 13:00:00', ARRAY['stripe-test@test.koso.app'])",
        )
        .execute(&pool)
        .await
        .unwrap();

        let user = User {
            email: "stripe-test@test.koso.app".to_string(),
            name: "IntegTesting DoNotDelete".to_string(),
            picture: "".to_string(),
            exp: 5,
        };
        let client = StripeClient {
            client: reqwest::Client::new(),
            secret_key: secrets::read_secret("stripe/secret_key").unwrap(),
        };
        let request = CreatePortalSessionRequest {
            return_url: "http://localhost/success".to_string(),
        };
        let Json(res) = handle_create_portal_session(
            Extension(user),
            Extension(&pool),
            Extension(client),
            Json(request),
        )
        .await
        .unwrap();
        assert!(!res.redirect_url.is_empty());
    }

    #[test_log::test(sqlx::test)]
    async fn update_subscription(pool: PgPool) {
        sqlx::query(
            "
            INSERT INTO users (email, name, picture, subscription_end_time, github_user_id)
            VALUES
                ('stripe-test@test.koso.app', 'IntegTesting DoNotDelete', '', null, null),
                ('user-1@test.koso.app', 'User One', '', TIMESTAMP '2040-01-20 13:00:00', null),
                ('user-2@test.koso.app', 'User Two', '', TIMESTAMP '2035-01-20 13:00:00', null),
                ('user-4@test.koso.app', 'User Four', '', TIMESTAMP '2040-01-20 13:00:00', null)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "
            INSERT INTO subscriptions (email, stripe_customer_id, seats, end_time, member_emails)
            VALUES
                ('stripe-test@test.koso.app', 'cus_SZSSBiHc9f8eQ6', 5, TIMESTAMP '2040-01-20 13:00:00', ARRAY['stripe-test@test.koso.app', 'user-1@test.koso.app','user-4@test.koso.app']),
                ('other-sub@test.koso.app', 'cus_foo', 5, TIMESTAMP '2035-01-20 13:00:00', ARRAY['user-2@test.koso.app','user-4@test.koso.app'])",
        )
        .execute(&pool)
        .await
        .unwrap();

        let user = User {
            email: "stripe-test@test.koso.app".to_string(),
            name: "IntegTesting DoNotDelete".to_string(),
            picture: "".to_string(),
            exp: 5,
        };

        // Remove user-1@test.koso.app
        // Add user-2@test.koso.app and user-3@test.koso.app
        let request = UpdateSubscriptionRequest {
            members: vec![
                "stripe-test@test.koso.app".to_string(),
                "stripe-test@test.koso.app".to_string(),
                "user-3@test.koso.app".to_string(),
                "user-2@test.koso.app".to_string(),
            ],
        };
        let Json(_) =
            handle_update_subscription(Extension(user.clone()), Extension(&pool), Json(request))
                .await
                .unwrap();

        let (_, members, _) = get_subscription_details(&user, &mut pool.begin().await.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            members,
            vec![
                "stripe-test@test.koso.app".to_string(),
                "user-2@test.koso.app".to_string(),
                "user-3@test.koso.app".to_string(),
            ]
        );

        assert_eq!(
            fetch_subscription_end_time("stripe-test@test.koso.app", &pool)
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            fetch_subscription_end_time("user-1@test.koso.app", &pool)
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            fetch_subscription_end_time("user-2@test.koso.app", &pool)
                .await
                .unwrap(),
            Some(
                DateTime::parse_from_rfc3339("2040-01-20 13:00:00z")
                    .unwrap()
                    .to_utc()
            )
        );
        assert_eq!(
            fetch_subscription_end_time("user-4@test.koso.app", &pool)
                .await
                .unwrap(),
            Some(
                DateTime::parse_from_rfc3339("2035-01-20 13:00:00z")
                    .unwrap()
                    .to_utc()
            )
        );
    }

    #[test_log::test(sqlx::test)]
    async fn test_handle_webhook(pool: PgPool) {
        sqlx::query(
            "
            INSERT INTO users (email, name, picture, subscription_end_time, github_user_id)
            VALUES ('stripe-test@test.koso.app', 'IntegTesting DoNotDelete', '', TIMESTAMP '2100-01-20 13:00:00', null)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "
            INSERT INTO subscriptions (email, stripe_customer_id, seats, end_time, member_emails)
            VALUES ('stripe-test@test.koso.app', 'cus_SZSSBiHc9f8eQ6', 5, TIMESTAMP '2100-01-20 13:00:00', ARRAY['stripe-test@test.koso.app']),
            ('other@test.koso.app', 'cus_SZSSBiHc9f8eQ6', 5, TIMESTAMP '2025-06-20 13:00:00', ARRAY['stripe-test@test.koso.app'])",
        )
        .execute(&pool)
        .await
        .unwrap();

        let user = User {
            email: "stripe-test@test.koso.app".to_string(),
            name: "IntegTesting DoNotDelete".to_string(),
            picture: "".to_string(),
            exp: 5,
        };
        let webhook_secret = WebhookSecret(Secret {
            data: "something".as_bytes().to_vec(),
        });
        let client = StripeClient {
            client: reqwest::Client::new(),
            secret_key: secrets::read_secret("stripe/secret_key").unwrap(),
        };
        let headers = HeaderMap::new();
        let body = Body::from(include_str!("../testdata/checkout.session.completed.json"));
        handle_webhook(
            Extension(webhook_secret),
            Extension(&pool),
            Extension(client),
            headers,
            body,
        )
        .await
        .unwrap();

        let (seats, members, end_time) =
            get_subscription_details(&user, &mut pool.begin().await.unwrap())
                .await
                .unwrap()
                .unwrap();
        assert_eq!(seats, 3);
        assert_eq!(members, vec!["stripe-test@test.koso.app".to_string(),]);
        assert!(end_time.timestamp() > 1750994651);
        assert!(end_time.timestamp() < 3486684251);

        let user_end_time = fetch_subscription_end_time("stripe-test@test.koso.app", &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user_end_time, end_time);
    }

    async fn fetch_subscription_end_time(
        email: &str,
        pool: &PgPool,
    ) -> Result<Option<DateTime<Utc>>> {
        let (end_time,): (Option<DateTime<Utc>>,) = sqlx::query_as(
            "
            SELECT subscription_end_time
            FROM users
            WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .context("Failed to query user subscription")?
        .context("User not found")?;
        Ok(end_time)
    }
}
