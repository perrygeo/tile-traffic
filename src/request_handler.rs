//! Handle the HTTP requests and emit metrics
//!
use crate::{statistics::RequestMetric, strategies::WebMapSession};
use log::{debug, error};
use reqwest::StatusCode;
use std::time::Instant;
use tokio::sync::mpsc;

/// Wraps the reqwest, gathers and emits stats
pub async fn request_handler(
    strategy: impl WebMapSession,
    seed: usize,
    tx_stats: mpsc::Sender<RequestMetric>,
) {
    let (url, tile) = strategy.make_url(seed);
    let start = Instant::now();
    let res = reqwest::get(&url).await;
    debug!("{:?}", res);
    let duration = start.elapsed();

    let metric = if let Ok(response) = res {
        // success!
        // Parse stats from response
        let status = response.status();
        let path = response.url().to_string();
        let headers = response.headers().clone();
        let content_type = headers.get("Content-Type");
        let content_length = if let Some(length) = response.content_length() {
            length
        } else {
            response.bytes().await.unwrap().len() as u64
        };

        RequestMetric::new(
            path,
            status,
            content_length,
            duration,
            content_type,
            tile.zoom,
        )
    } else {
        // failed :-(
        // TODO inspect the error and make sure that
        // 502 Bad Gateway is an appropriate status code
        RequestMetric::new(url, StatusCode::BAD_GATEWAY, 0, duration, None, tile.zoom)
    };

    tx_stats.send(metric).await.unwrap();
}
