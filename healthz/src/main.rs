use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
struct Chat {
    ok: bool,
    result: ChatResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatResult {
    title: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    healthz_status: bool,
    last_update: u128,
}

struct Telegram {
    token: String,
    chat_id: String,
}

fn client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    ClientBuilder::new(Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

impl Telegram {
    async fn get_chat(&self) -> Result<Chat> {
        Ok(client()
            .get(format!(
                "https://api.telegram.org/bot{}/getChat?chat_id={}",
                &self.token, &self.chat_id
            ))
            .send()
            .await?
            .json()
            .await?)
    }

    async fn get_status(&self) -> Result<Status> {
        let chat = self.get_chat().await?;
        let status: Status = serde_json::from_str(&chat.result.description)?;
        Ok(status)
    }

    async fn set_status(&self, status: &Status) -> Result<()> {
        let serialized_status = serde_json::to_string(&status)?;
        let resp = client()
            .get(format!(
                "https://api.telegram.org/bot{}/setChatDescription?chat_id={}&description={}",
                &self.token, &self.chat_id, serialized_status
            ))
            .send()
            .await?
            .text()
            .await?;
        println!("set_status result: {resp}");
        Ok(())
    }

    async fn send_message(&self, message: &str) -> Result<()> {
        let resp = client()
            .get(format!(
                "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
                &self.token, &self.chat_id, message
            ))
            .send()
            .await?
            .text()
            .await?;
        println!("send_message result: {resp}");
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthZ {
    status: String,
}

async fn check_healthz(url: &str) -> Result<HealthZ> {
    let healthz: HealthZ = client().get(url).send().await?.json().await?;
    Ok(healthz)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    url: String,
    #[arg(short, long, required = true)]
    token: String,
    #[arg(short, long, required = true)]
    chat_id: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let telegram = Telegram {
        token: cli.token.to_string(),
        chat_id: cli.chat_id.to_string(),
    };

    let prev_status = match telegram.get_status().await {
        Ok(s) => s,
        Err(e) => {
            println!("Resetting due to load status failure: {e}");
            Status {
                healthz_status: true,
                last_update: 0,
            }
        }
    };

    let healthz_status = check_healthz(&cli.url).await.is_ok();
    let last_update = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let curr_status = Status {
        healthz_status,
        last_update,
    };
    match telegram.set_status(&curr_status).await {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to update healthz status: {e}");
        }
    };

    if curr_status.healthz_status != prev_status.healthz_status {
        let resp = if curr_status.healthz_status {
            telegram.send_message("✅ Koso is up!")
        } else {
            telegram.send_message("❌ Koso is down!")
        }
        .await;
        match resp {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to send update: {e}");
            }
        }
    }
}
