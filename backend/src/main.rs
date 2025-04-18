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
    tokio::join!(
        run_server(shutdown_signal.clone()),
        run_metrics_server(shutdown_signal.clone()),
        run_telegram_server(shutdown_signal.clone()),
        signal_shutdown(shutdown_signal.clone()),
    );
}

async fn run_server(shutdown_signal: CancellationToken) {
    tokio::spawn(async move {
        let (_port, serve) = server::start_main_server(server::Config {
            shutdown_signal: shutdown_signal.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
        serve.await.unwrap().unwrap();
    })
    .await
    .unwrap();
}

async fn run_metrics_server(shutdown_signal: CancellationToken) {
    tokio::spawn(async move {
        let (_port, serve) = metrics_server::start_metrics_server(metrics_server::Config {
            shutdown_signal: shutdown_signal.clone(),
            ..Default::default()
        })
        .await
        .unwrap();
        serve.await.unwrap().unwrap();
    })
    .await
    .unwrap()
}

async fn run_telegram_server(shutdown_signal: CancellationToken) {
    tokio::spawn(async move {
        notifiers::telegram::start_telegram_server(shutdown_signal.clone())
            .await
            .unwrap();
    })
    .await
    .unwrap()
}

// This function waits for a shutdown signal (e.g. ctrl-c, SIGTERM)
// and then cancels the provided CancellationToken in order
// to enable graceful shutdown.
async fn signal_shutdown(shutdown_signal: CancellationToken) {
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

    // Wait for one of the signals to fire.
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    // Initiate shutdown.
    shutdown_signal.cancel();
}
