use anyhow::{Context as _, Result};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod debug;
mod healthz;
mod mcp;
mod metrics_server;
mod notifiers;
mod plugins;
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
    tokio::join!(
        async { run_server(shutdown_signal.clone()).await.unwrap() },
        async { run_metrics_server(shutdown_signal.clone()).await.unwrap() },
        async { signal_shutdown(shutdown_signal.clone()).await.unwrap() },
    );
}

async fn run_server(shutdown_signal: CancellationToken) -> Result<()> {
    tokio::spawn(async move {
        let (_port, serve) = server::start_main_server(server::Config {
            shutdown_signal: shutdown_signal.clone(),
            ..Default::default()
        })
        .await?;
        serve.await??;
        Ok::<(), anyhow::Error>(())
    })
    .await?
}

async fn run_metrics_server(shutdown_signal: CancellationToken) -> Result<()> {
    tokio::spawn(async move {
        let (_port, serve) = metrics_server::start_metrics_server(metrics_server::Config {
            shutdown_signal: shutdown_signal.clone(),
            ..Default::default()
        })
        .await?;
        serve.await??;
        Ok::<(), anyhow::Error>(())
    })
    .await?
}

// This function waits for a shutdown signal (e.g. ctrl-c, SIGTERM)
// and then cancels the provided CancellationToken in order
// to enable graceful shutdown.
async fn signal_shutdown(shutdown_signal: CancellationToken) -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .context("failed to install Ctrl+C handler")?;
        tracing::info!("Terminating with ctrl-c...");
        Ok::<(), anyhow::Error>(())
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .context("failed to install signal handler")?
            .recv()
            .await;
        tracing::info!("Terminating...");
        Ok(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<()>>();

    // Wait for one of the signals to fire.
    tokio::select! {
        res = ctrl_c => {res},
        res = terminate => {res},
    }?;

    // Initiate shutdown.
    shutdown_signal.cancel();

    Ok(())
}
