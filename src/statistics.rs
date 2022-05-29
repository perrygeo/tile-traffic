use log::{debug, info};
use tokio::sync::mpsc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RequestStat {
    path: String,
    status: reqwest::StatusCode,
    content_length: u64,
    duration: std::time::Duration,
    content_type: String,
}

impl RequestStat {
    pub fn new(
        path: String,
        status: reqwest::StatusCode,
        content_length: u64,
        duration: std::time::Duration,
        content_type: Option<&reqwest::header::HeaderValue>,
    ) -> Self {
        let content_type = if let Some(cty) = content_type {
            cty.to_str().unwrap().to_string()
        } else {
            String::from("")
        };

        RequestStat {
            path,
            status,
            content_length,
            duration,
            content_type,
        }
    }
}

/// An "actor" to handle the stats messages
pub async fn stats_actor(rx: mpsc::Receiver<RequestStat>) {
    let mut rx = rx;
    let mut count = 0;
    let mut cumulative_duration = 0.0;
    let mut cumulative_length = 0;
    // TODO
    // count by status code
    // count by content type
    // size vs response time
    // histogram of response time
    // histogram of size
    // response time vs lat
    // response time vs z
    while let Some(s) = rx.recv().await {
        count += 1;
        cumulative_duration += s.duration.as_secs_f64();
        cumulative_length += s.content_length;
        debug!("{:?}", s);
    }
    let mean_duration = cumulative_duration / count as f64;
    let mean_length = cumulative_length / count;
    info!("Count: {} tiles", count);
    info!("Mean Duration: {} ms", mean_duration);
    info!("Mean Content Length: {} bytes", mean_length);
}
