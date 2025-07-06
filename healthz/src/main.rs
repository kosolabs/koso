use anyhow::Result;
use axum::{Extension, Json, Router, extract::Query, http::StatusCode, routing::get};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize, Debug)]
struct TelegramResponse<T> {
    ok: bool,
    result: T,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelegramChat {
    id: i64,
    #[serde(rename = "type")]
    chat_type: String,
    title: Option<String>,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TelegramMessage {
    message_id: i32,
    date: i64,
    chat: TelegramChat,
    text: Option<String>,
}

#[derive(Clone)]
struct TelegramClient {
    client: Client,
    token: String,
}

impl TelegramClient {
    fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            token: env::var("TELOXIDE_TOKEN")?,
        })
    }

    async fn get_chat(&self, chat_id: i64) -> Result<TelegramChat> {
        let url = format!("https://api.telegram.org/bot{}/getChat", self.token);

        let response: TelegramResponse<TelegramChat> = self
            .client
            .get(&url)
            .query(&[("chat_id", chat_id)])
            .send()
            .await?
            .json()
            .await?;

        if response.ok {
            Ok(response.result)
        } else {
            Err(anyhow::anyhow!(
                "Telegram API error: {:?}",
                response.description
            ))
        }
    }

    async fn send_message(&self, chat_id: i64, text: &str) -> Result<TelegramMessage> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        let response: TelegramResponse<TelegramMessage> = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text
            }))
            .send()
            .await?
            .json()
            .await?;

        if response.ok {
            Ok(response.result)
        } else {
            Err(anyhow::anyhow!(
                "Telegram API error: {:?}",
                response.description
            ))
        }
    }

    async fn set_chat_description(&self, chat_id: i64, description: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/setChatDescription",
            self.token
        );

        let response: TelegramResponse<bool> = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "description": description
            }))
            .send()
            .await?
            .json()
            .await?;

        if response.ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Telegram API error: {:?}",
                response.description
            ))
        }
    }

    async fn pin_chat_message(
        &self,
        chat_id: i64,
        message_id: i32,
        disable_notification: bool,
    ) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/pinChatMessage", self.token);

        let response: TelegramResponse<bool> = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "disable_notification": disable_notification
            }))
            .send()
            .await?
            .json()
            .await?;

        if response.ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Telegram API error: {:?}",
                response.description
            ))
        }
    }

    async fn unpin_all_chat_messages(&self, chat_id: i64) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/unpinAllChatMessages",
            self.token
        );

        let response: TelegramResponse<bool> = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id
            }))
            .send()
            .await?
            .json()
            .await?;

        if response.ok {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Telegram API error: {:?}",
                response.description
            ))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Status {
    healthz_status: bool,
    last_update: u128,
}

fn get_status(chat: &TelegramChat) -> Status {
    let Some(description) = &chat.description else {
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
    let client = ClientBuilder::new(Client::new())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build();

    let healthz: HealthZ = client
        .get(url)
        .header("koso-client-version", "healthz-binary")
        .send()
        .await?
        .json()
        .await?;
    tracing::info!("check_healthz: {healthz:?}");
    Ok(healthz)
}

async fn check_and_notify(client: TelegramClient, url: &str, chat_id: i64) -> Result<Status> {
    tracing::info!("Checking status of: {}", url);

    let chat = client.get_chat(chat_id).await?;
    let prev_status = get_status(&chat);

    let healthz_status = match check_healthz(url).await {
        Ok(_) => true,
        Err(error) => {
            tracing::error!("Healthz check failed: {error:?}");
            false
        }
    };
    let last_update = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let curr_status = Status {
        healthz_status,
        last_update,
    };

    let serialized_status = serde_json::to_string(&curr_status)?;
    client
        .set_chat_description(chat_id, &serialized_status)
        .await?;

    tracing::info!("prev: {prev_status:?}, curr: {curr_status:?}");
    if curr_status.healthz_status == prev_status.healthz_status {
        return Ok(curr_status);
    }

    let message_text = if curr_status.healthz_status {
        "✅ Koso is up!"
    } else {
        "❌ Koso is down!"
    };

    let message = client.send_message(chat_id, message_text).await?;

    client.unpin_all_chat_messages(chat_id).await?;
    client
        .pin_chat_message(chat_id, message.message_id, true)
        .await?;

    Ok(curr_status)
}

#[derive(Deserialize)]
struct CheckAndNotify {
    url: String,
    #[serde(rename = "chat-id")]
    chat_id: i64,
}

async fn handle(
    Extension(client): Extension<TelegramClient>,
    Query(params): Query<CheckAndNotify>,
) -> Result<Json<Status>, StatusCode> {
    match check_and_notify(client, &params.url, params.chat_id).await {
        Ok(status) => Ok(Json(status)),
        Err(error) => {
            tracing::error!("Failed to check healthz: {error:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let telegram_client = TelegramClient::new().unwrap();
    let app = Router::new()
        .route("/", get(handle))
        .layer(Extension(telegram_client));
    let port = env::var("PORT").unwrap_or("8000".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    tracing::info!("Listening on {:?}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
