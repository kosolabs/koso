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
    tracing::info!("Using koso settings: {:?}", settings::settings());

    let (_main_server, _metrics_server, _telegram_server) = tokio::join!(
        async {
            let (_port, serve) = server::start_main_server(server::Config::default()).await;
            serve.await.unwrap();
        },
        async {
            let (_port, serve) =
                metrics_server::start_metrics_server(metrics_server::Config::default()).await;
            serve.await.unwrap();
        },
        async {
            notifiers::telegram::start_telegram_server().await.unwrap();
        },
    );
}
