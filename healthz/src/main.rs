use anyhow::Result;
use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};
use teloxide::{prelude::*, types::Chat};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Status {
    healthz_status: bool,
    last_update: u128,
}

fn client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

fn get_status(chat: &Chat) -> Status {
    let Some(description) = chat.description() else {
        tracing::info!("Failed to load chat description. Resetting.");
        return Status::default();
    };

    let Ok(status) = serde_json::from_str(description) else {
        tracing::info!("Failed to parse status from description. Resetting.");
        return Status::default();
    };

    status
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthZ {
    status: String,
}

async fn check_healthz(url: &str) -> Result<HealthZ> {
    let healthz: HealthZ = client()
        .get(url)
        .header("koso-client-version", "healthz-binary")
        .send()
        .await?
        .json()
        .await?;
    tracing::info!("check_healthz: {healthz:?}");
    Ok(healthz)
}

async fn check_and_notify(url: &str, chat_id: ChatId) -> Result<Status> {
    tracing::info!("Checking status of: {}", url);

    let bot = Bot::from_env();

    let chat = bot.get_chat(chat_id).await?;
    let prev_status = get_status(&chat);

    let healthz_status = check_healthz(url).await.is_ok();
    let last_update = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let curr_status = Status {
        healthz_status,
        last_update,
    };

    let serialized_status = serde_json::to_string(&curr_status)?;
    bot.set_chat_description(chat_id)
        .description(serialized_status)
        .await?;

    tracing::info!("prev: {prev_status:?}, curr: {curr_status:?}");
    if curr_status.healthz_status == prev_status.healthz_status {
        return Ok(curr_status);
    }

    let message = if curr_status.healthz_status {
        bot.send_message(chat_id, "✅ Koso is up!")
    } else {
        bot.send_message(chat_id, "❌ Koso is down!")
    }
    .await?;

    bot.unpin_all_chat_messages(chat_id).await?;

    bot.pin_chat_message(chat_id, message.id)
        .disable_notification(true)
        .await?;

    Ok(curr_status)
}

#[derive(Deserialize)]
struct CheckAndNotify {
    url: String,
    #[serde(rename = "chat-id")]
    chat_id: i64,
}

async fn handle(Query(params): Query<CheckAndNotify>) -> Result<Json<Status>, StatusCode> {
    match check_and_notify(&params.url, ChatId(params.chat_id)).await {
        Ok(status) => Ok(Json(status)),
        Err(error) => {
            tracing::error!("Failed to check healthz: {:?}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let _ = Bot::from_env();
    let app = Router::new().route("/", get(handle));
    let port = env::var("PORT").unwrap_or("8000".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    tracing::info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
