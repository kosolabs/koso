use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::{prelude::*, types::Chat};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    url: String,
    #[arg(short, long, required = true)]
    chat_id: i64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let chat_id = cli.chat_id.to_string();

    tracing::info!("Checking status of: {}", &cli.url);

    let bot = Bot::from_env();

    let chat = bot
        .get_chat(chat_id.to_string())
        .await
        .expect("Failed to get chat");
    let prev_status = get_status(&chat);

    let healthz_status = check_healthz(&cli.url).await.is_ok();
    let last_update = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    let curr_status = Status {
        healthz_status,
        last_update,
    };

    let serialized_status =
        serde_json::to_string(&curr_status).expect("Failed to serialize updated status");
    bot.set_chat_description(chat_id.to_string())
        .description(serialized_status)
        .await
        .expect("Failed to update status");

    tracing::info!("prev: {prev_status:?}, curr: {curr_status:?}");
    if curr_status.healthz_status == prev_status.healthz_status {
        return;
    }

    let message = if curr_status.healthz_status {
        bot.send_message(chat_id.to_string(), "✅ Koso is up!")
    } else {
        bot.send_message(chat_id.to_string(), "❌ Koso is down!")
    }
    .await
    .expect("Failed to send update");

    bot.unpin_all_chat_messages(chat_id.to_string())
        .await
        .expect("Failed to unpin all chat messages");

    bot.pin_chat_message(chat_id.to_string(), message.id)
        .disable_notification(true)
        .await
        .expect("Failed to pin updated message");
}
