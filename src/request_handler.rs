//! Handle the HTTP requests and emit metrics
//!
use crate::{statistics::RequestMetric, strategies::WebMapSession};
use log::{debug, error};
use std::time::Instant;
use tokio::sync::mpsc;

/// Wraps the reqwest, gathers and emits stats
pub async fn request_handler(
    strategy: impl WebMapSession,
    seed: usize,
    tx_stats: mpsc::Sender<RequestMetric>,
) {
    let url = strategy.make_url(seed);
    let start = Instant::now();
    let res = reqwest::get(url).await;
    debug!("{:?}", res);
    let duration = start.elapsed();

    if let Ok(response) = res {
        // Parse stats from response
        let status = response.status();
        let path = response.url().to_string();
        let headers = response.headers().clone();
        let content_length = if let Some(length) = response.content_length() {
            length
        } else {
            response.bytes().await.unwrap().len() as u64
        };
        let content_type = headers.get("Content-Type");

        // Send stats
        let stat = RequestMetric::new(path, status, content_length, duration, content_type);
        tx_stats.send(stat).await.unwrap();
    } else {
        // TODO send failed metric
        // let stat = RequestMetric::new(path, status, None, duration, None);
        // tx_stats.send(stat).await.unwrap();
        error!("{:?}", res);
    };
}
