//! Gather stats from HTTP requests, summarize, and update the UI state
//!
use log::info;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::tui;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RequestMetric {
    path: String,
    status: reqwest::StatusCode,
    content_length: u64,
    duration: std::time::Duration,
    content_type: String,
}

impl RequestMetric {
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

        RequestMetric {
            path,
            status,
            content_length,
            duration,
            content_type,
        }
    }
}

fn mean(xs: &[f64]) -> f64 {
    let count = xs.len() as f64;
    let sum: f64 = xs.iter().sum();
    sum / count
}

fn incr_count(mut hm: HashMap<String, usize>, k: String) -> HashMap<String, usize> {
    let val = if let Some(v) = hm.get(&k) { v + 1 } else { 1 };
    hm.insert(k, val);
    hm
}

/// handles the incoming request metrics and calculates stats.
pub async fn stats_actor(rx: mpsc::Receiver<RequestMetric>) {
    let mut rx = rx;
    let mut count = 0;
    let mut response_times = Vec::new();
    let mut response_sizes = Vec::new();
    let mut status_codes = HashMap::new();
    let mut content_types = HashMap::new();
    // TODO let mut zoom_levels = HashMap::new();

    while let Some(s) = rx.recv().await {
        count += 1;

        response_times.push((s.duration.as_secs_f64() * 1000.).round());
        response_sizes.push((s.content_length as f64 / 1000.).round());
        status_codes = incr_count(status_codes, s.status.to_string());
        content_types = incr_count(content_types, s.content_type.to_string());

        // this blocks the main thread but makes the borrow checker happy
        // try with spawn_blocking and you have to clone the Vec hmmm....
        // TODO rather than passing references
        // maybe state.render_text() -> string
        // then give ownership of the output string to tui?
        if count % 3 == 0 {
            tui::draw(tui::TuiState {
                response_times: &response_times,
                response_sizes: &response_sizes,
                status_codes: &status_codes,
                content_types: &content_types,
            });
        }
    }

    // Finalize
    tui::draw(tui::TuiState {
        response_times: &response_times,
        response_sizes: &response_sizes,
        status_codes: &status_codes,
        content_types: &content_types,
    });
    let mean_size: f64 = mean(&response_sizes);
    let mean_time: f64 = mean(&response_times);
    info!("Count: {} tiles", count);
    info!("Mean Duration: {:0.2} ms", mean_time);
    info!("Mean Content Length: {:0.2} kB", mean_size);
}
