use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod healthz;
mod metrics_server;
mod notifiers;
mod plugins;
mod postgres;
mod secrets;
mod server;
mod settings;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    let settings = settings::settings();
    tracing::info!("Using koso settings: {settings:?}");

    let shutdown_signal = CancellationToken::new();

    let main_server = {
        let config = server::Config {
            shutdown_signal: shutdown_signal.clone(),
            ..Default::default()
        };
        async {
            let (_port, serve) = server::start_main_server(config).await.unwrap();
            serve.await.unwrap();
        }
    };
    let metrics_server = {
        let config = metrics_server::Config {
            shutdown_signal: shutdown_signal.clone(),
            ..Default::default()
        };
        async {
            let (_port, serve) = metrics_server::start_metrics_server(config).await.unwrap();
            serve.await.unwrap();
        }
    };
    let telegram_server = {
        let shutdown_signal: CancellationToken = shutdown_signal.clone();
        async {
            notifiers::telegram::start_telegram_server(shutdown_signal)
                .await
                .unwrap();
        }
    };

    let (_main_server, _metrics_server, _telegram_server) =
        tokio::join!(main_server, metrics_server, telegram_server,);
}
