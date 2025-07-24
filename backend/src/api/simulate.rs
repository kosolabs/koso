use anyhow::anyhow;
use axum::{body::Body, response::Response};
use futures::{StreamExt, stream};

use crate::api::ApiResult;

pub(super) async fn simulate(method: &str) -> ApiResult<Response> {
    let data = if method == "summarize" {
        include_str!("simulations/summarize.txt")
    } else if method == "breakdown" {
        include_str!("simulations/breakdown.txt")
    } else if method == "context" {
        include_str!("simulations/context.txt")
    } else {
        return Err(anyhow!("Invalid method").into());
    };
    let chunks = data
        .split_inclusive("\n\n")
        .map(|chunk| chunk.as_bytes().to_vec())
        .collect::<Vec<Vec<u8>>>();

    let test_stream = stream::iter(chunks).then(move |chunk| async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok::<_, std::io::Error>(chunk)
    });

    Ok(Response::builder()
        .status(200)
        .body(Body::from_stream(test_stream))?)
}
