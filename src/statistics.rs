//! Gather stats from HTTP requests, summarize, and update the UI state
//!
use log::info;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::tui::TuiState;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RequestMetric {
    path: String,
    status: reqwest::StatusCode,
    content_length: u64,
    duration: std::time::Duration,
    content_type: String,
    zoom: u32,
}

impl RequestMetric {
    pub fn new(
        path: String,
        status: reqwest::StatusCode,
        content_length: u64,
        duration: std::time::Duration,
        content_type: Option<&reqwest::header::HeaderValue>,
        zoom: u32,
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
            zoom,
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
pub async fn stats_actor(mut rx: mpsc::Receiver<RequestMetric>, mut state: TuiState) {
    let mut count = 0;

    // Loop: Receive messages on a channel and update the state
    while let Some(s) = rx.recv().await {
        count += 1;
        state
            .response_times
            .push((s.duration.as_secs_f64() * 1000.).round());
        state
            .response_sizes
            .push((s.content_length as f64 / 1000.).round());
        state.status_codes = incr_count(state.status_codes, s.status.to_string());
        state.content_types = incr_count(state.content_types, s.content_type.to_string());
        state.zoom_levels = incr_count(state.zoom_levels, s.zoom.to_string());

        // this blocks the tokio thread
        // TODO spawn_blocking? So far it's fast enough to not matter.
        state.draw();
    }

    // No more incoming requests, someone dropped the tx end
    // Finalize outputs
    state.draw();
    info!("Count: {} tiles", count);
    let mean_time: f64 = mean(&state.response_times);
    info!("Mean response time: {:0.2} ms", mean_time);
    let mean_size: f64 = mean(&state.response_sizes);
    info!("Mean content length: {:0.2} kB", mean_size);
}
