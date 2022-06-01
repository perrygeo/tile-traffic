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
pub async fn stats_actor(rx: mpsc::Receiver<RequestMetric>) {
    let mut rx = rx;
    let mut count = 0;

    let mut state = TuiState::default();

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

        // this blocks the tokio thread, TODO spawn blocking?
        state.draw();
    }

    // Finalize
    state.draw();
    info!("Count: {} tiles", count);
    let mean_time: f64 = mean(&state.response_times);
    info!("Mean Duration: {:0.2} ms", mean_time);
    let mean_size: f64 = mean(&state.response_sizes);
    info!("Mean Content Length: {:0.2} kB", mean_size);
}
