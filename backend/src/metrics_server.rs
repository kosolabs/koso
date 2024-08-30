use axum::{routing::get, Router};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use std::{future::ready, net::SocketAddr};
use tokio::{sync::oneshot::Receiver, task::JoinHandle};

use crate::server::shutdown_signal;

#[derive(Default)]
pub struct Config {
    pub port: Option<u16>,
    pub shutdown_signal: Option<Receiver<()>>,
}

/// Starts a prometheus metrics server and returns a future that completes on termination.
#[tracing::instrument(skip(config))]
pub async fn start_metrics_server(config: Config) -> (SocketAddr, JoinHandle<()>) {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
    ];
    let recorder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap();

    let app = Router::new().route("/metrics", get(move || ready(recorder.render())));
    // The `/metrics` endpoint should not be publicly available.
    let port = config.port.unwrap_or(3001);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();

    let addr = listener.local_addr().unwrap();
    let serve = tokio::spawn(async {
        tracing::debug!(
            "metrics server listening on {}",
            listener.local_addr().unwrap()
        );
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal("metrics server", config.shutdown_signal))
            .await
            .unwrap();
    });

    return (addr, serve);
}

#[cfg(test)]
mod tests {
    use crate::metrics_server;
    use reqwest::{Client, StatusCode};
    use tokio::sync::oneshot::channel;

    #[test_log::test(tokio::test)]
    async fn metrics_server_test() -> anyhow::Result<()> {
        let (closer, close_signal) = channel::<()>();
        let (addr, serve) = metrics_server::start_metrics_server(metrics_server::Config {
            port: Some(0),
            shutdown_signal: Some(close_signal),
        })
        .await;

        let client = Client::default();
        let res = client
            .get(&format!("http://{addr}/metrics"))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);

        if let Err(e) = closer.send(()) {
            panic!("Error sending close signal: {e:?}");
        }
        if let Err(e) = serve.await {
            panic!("Server failed: {e:?}");
        }
        Ok(())
    }
}
