use crate::{
    flags::is_dev,
    secrets::{read_secret, Secret},
};
use anyhow::Result;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{MessageFilterExt, UpdateFilterExt},
    dptree,
    prelude::{Dispatcher, Requester},
    types::{Message, Update, User},
    Bot,
};

fn bot_from_secrets() -> Result<Bot> {
    let secret: Secret<String> = read_secret("telegram/token")?;
    Ok(Bot::new(secret.data))
}

fn encoding_key_from_secrets() -> EncodingKey {
    // Failing to init the encoding key will panic to prevent the server from starting
    let secret: Secret<String> = read_secret("koso/hmac").unwrap();
    EncodingKey::from_base64_secret(&secret.data).unwrap()
}

pub(crate) async fn start_telegram_server() -> Result<()> {
    let bot = match bot_from_secrets() {
        Ok(bot) => bot,
        Err(error) => {
            if is_dev() {
                tracing::warn!("Telegram bot not started because token is not set.");
                return Ok(());
            } else {
                return Err(error);
            }
        }
    };
    let key = encoding_key_from_secrets();
    let schema = Update::filter_message()
        .filter_map(|update: Update| update.from().cloned())
        .branch(Message::filter_text().endpoint(process_text_message));
    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![key])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    chat_id: u64,
}

async fn process_text_message(
    bot: Bot,
    key: EncodingKey,
    user: User,
    message_text: String,
) -> Result<()> {
    let claims = Claims { chat_id: user.id.0 };
    let token = encode(&Header::default(), &claims, &key)?;
    tracing::info!("Received from {user:?}: {message_text}");
    bot.send_message(user.id, format!("Login to Koso with this token: {token}"))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::notify::encoding_key_from_secrets;

    #[test_log::test(tokio::test)]
    async fn encode() {
        encoding_key_from_secrets();
    }
}
