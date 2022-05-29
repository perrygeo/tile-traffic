use log::{debug, error};
use std::time::Instant;
use tokio::sync::mpsc;

use crate::statistics::RequestStat;
use crate::strategies::make_url;

/// Wraps the reqwest, gathers and emits stats
pub async fn request_handler(template: String, seed: usize, tx_stats: mpsc::Sender<RequestStat>) {
    let url = make_url(template, seed);
    debug!("initiating request for {}", url);
    let start = Instant::now();
    let res = reqwest::get(url).await;
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
        let stat = RequestStat::new(path, status, content_length, duration, content_type);
        tx_stats.send(stat).await.unwrap();
    } else {
        error!("{:?}", res);
    };
}
