use crate::{
    api::{
        ApiResult, bad_request_error,
        billing::{
            model::{
                CreateCheckoutSessionRequest, CreateCheckoutSessionResponse,
                CreatePortalSessionRequest, CreatePortalSessionResponse, UpdateSubscriptionRequest,
                UpdateSubscriptionResponse,
            },
            stripe::{KosoMetadata, StripeClient},
            webhook::WebhookSecret,
        },
        google::{self, User},
        not_found_error,
    },
    secrets::{self},
    settings::settings,
};
use anyhow::{Context, Result};
use axum::middleware;
use axum::routing::put;
use axum::{Extension, Json, Router, routing::post};
use sqlx::PgPool;
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
            post(create_checkout_session),
        )
        .route("/stripe/create-portal-session", post(create_portal_session))
        .route("/stripe/subscription", put(update_subscription))
        .layer((middleware::from_fn(google::authenticate),))
        // The webhook endpoint is invoked by Stripe and not users. Don't authenticate using Google.
        .route("/stripe/webhook", post(webhook::handle_webhook))
        .layer((Extension(client),))
        .layer((Extension(webhook_secret),)))
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
async fn create_portal_session(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
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
async fn update_subscription(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Json(request): Json<UpdateSubscriptionRequest>,
) -> ApiResult<Json<UpdateSubscriptionResponse>> {
    let mut members = request
        .members
        .into_iter()
        .map(|e| (e.to_lowercase(), true))
        .collect::<HashMap<String, bool>>()
        .into_keys()
        .collect::<Vec<String>>();
    members.sort();
    if !members.contains(&user.email) {
        return Err(bad_request_error(
            "MISSING_SELF",
            "Members must include owner",
        ));
    }

    let mut txn = pool.begin().await?;
    let res: Option<(i32,)> = sqlx::query_as(
        "
        SELECT seats
        FROM subscriptions
        WHERE email=$1",
    )
    .bind(&user.email)
    .bind(&members)
    .fetch_optional(&mut *txn)
    .await
    .context("Failed to fetch seats")?;
    let Some((seats,)) = res else {
        return Err(not_found_error("NOT_FOUND", "Subscription not found"));
    };
    if members.len() > usize::try_from(seats)? {
        return Err(bad_request_error(
            "TOO_MANY_MEMBERS",
            &format!("Tried to put {} members in {seats} seats", members.len()),
        ));
    }

    sqlx::query(
        "
        UPDATE subscriptions
        SET member_emails=$2
        WHERE email=$1",
    )
    .bind(&user.email)
    .bind(members)
    .execute(&mut *txn)
    .await
    .context("Failed to upsert subscription")?;

    txn.commit().await?;
    Ok(Json(UpdateSubscriptionResponse {}))
}

mod stripe {
    use crate::secrets::Secret;
    use anyhow::{Context, Result, anyhow};
    use reqwest::IntoUrl;
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

        pub(super) async fn get_subscription(&self, id: &str) -> Result<Subscription> {
            self.get(format!("https://api.stripe.com/v1/subscriptions/{}", id))
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
        http::request::Parts,
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
        skip(webhook_secret, pool, client, parts, body),
        fields(stripe_event, stripe_event_id)
    )]
    pub(super) async fn handle_webhook(
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
}

pub(crate) mod model {
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
}
