use anyhow::{Context, Result};
use tokio::signal;
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
    let (_shutdown, _main_server, _metrics_server, _telegram_server) = tokio::join!(
        async {
            signal_shutdown(shutdown_signal.clone()).await.unwrap();
        },
        async {
            let (_port, serve) = server::start_main_server(server::Config {
                shutdown_signal: shutdown_signal.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
            serve.await.unwrap().unwrap();
        },
        async {
            let (_port, serve) = metrics_server::start_metrics_server(metrics_server::Config {
                shutdown_signal: shutdown_signal.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
            serve.await.unwrap().unwrap();
        },
        async {
            notifiers::telegram::start_telegram_server(shutdown_signal.clone())
                .await
                .unwrap();
        },
    );
}

// This function waits for a shutdown signal (e.g. ctrl-c, SIGTERM)
// and then cancels the provided CancellationToken in order
// to enable graceful shutdown.
async fn signal_shutdown(shutdown_signal: CancellationToken) -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        tracing::info!("Terminating with ctrl-c...");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        tracing::info!("Terminating...");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::spawn(async move {
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        shutdown_signal.cancel();
    })
    .await
    .context("Shutdown signal task failed")
}
