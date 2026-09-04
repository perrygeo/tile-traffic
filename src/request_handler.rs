//! Handle the HTTP requests and emit metrics
//!
use log::{debug, error};
use reqwest::StatusCode;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::statistics::{RequestMetric, StatsEvent, MISSING_HEADER_VALUE};
use crate::strategies::WebMapSession;

fn header_values(
    headers: &reqwest::header::HeaderMap,
    names: &[String],
) -> HashMap<String, String> {
    names
        .iter()
        .map(|name| {
            let value = headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .unwrap_or(MISSING_HEADER_VALUE);
            (name.clone(), value.to_string())
        })
        .collect()
}

/// Invokes the WebMapSession, performing the request,
/// gathering and emiting metrics to a channel
pub async fn request_handler(
    strategy: impl WebMapSession,
    seed: usize,
    tx_stats: mpsc::Sender<StatsEvent>,
    header_names: &[String],
) {
    let (url, tile) = strategy.make_url(seed);
    tx_stats.send(StatsEvent::Url(url.clone())).await.unwrap();
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
            header_values(&headers, header_names),
        )
    } else {
        // failed :-(
        error!("{:?}", res);
        RequestMetric::new(
            url,
            StatusCode::from_u16(111).unwrap(),
            0,
            duration,
            None,
            tile.zoom,
            header_names
                .iter()
                .map(|name| (name.clone(), MISSING_HEADER_VALUE.to_string()))
                .collect(),
        )
    };

    tx_stats.send(StatsEvent::Metric(metric)).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statistics::MISSING_HEADER_VALUE;
    use reqwest::header::HeaderMap;

    #[test]
    fn header_values_extracts_present_and_missing() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Cache", reqwest::header::HeaderValue::from_static("HIT"));
        let names = vec!["X-Cache".to_string(), "X-Missing".to_string()];

        let values = header_values(&headers, &names);

        assert_eq!(values.get("X-Cache").map(String::as_str), Some("HIT"));
        assert_eq!(
            values.get("X-Missing").map(String::as_str),
            Some(MISSING_HEADER_VALUE)
        );
    }

    #[test]
    fn header_values_treats_non_utf8_as_missing() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Binary",
            reqwest::header::HeaderValue::from_bytes(&[0xff]).unwrap(),
        );
        let names = vec!["X-Binary".to_string()];

        let values = header_values(&headers, &names);

        assert_eq!(
            values.get("X-Binary").map(String::as_str),
            Some(MISSING_HEADER_VALUE)
        );
    }
}
