//! Gather stats from HTTP requests, summarize, and update the UI state
//!
use std::collections::HashMap;
use std::fmt::Display;
use tokio::sync::mpsc;

use crate::tui::TuiState;

pub const MISSING_HEADER_VALUE: &str = "(missing)";

const MAX_URLS: usize = 50;

#[derive(Debug)]
pub enum StatsEvent {
    Url(String),
    Metric(RequestMetric),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RequestMetric {
    path: String,
    status: reqwest::StatusCode,
    content_length: u64,
    duration: std::time::Duration,
    content_type: String,
    zoom: u32,
    header_values: HashMap<String, String>,
}

impl RequestMetric {
    pub fn new(
        path: String,
        status: reqwest::StatusCode,
        content_length: u64,
        duration: std::time::Duration,
        content_type: Option<&reqwest::header::HeaderValue>,
        zoom: u32,
        header_values: HashMap<String, String>,
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
            header_values,
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

fn incr_count_u32(mut hm: HashMap<u32, usize>, k: u32) -> HashMap<u32, usize> {
    let val = if let Some(v) = hm.get(&k) { v + 1 } else { 1 };
    hm.insert(k, val);
    hm
}

#[derive(Debug)]
pub struct Summary {
    pub tiles: usize,
    pub mean_response_time_ms: f64,
    pub mean_content_length_kb: f64,
    pub status_codes: HashMap<String, usize>,
    pub content_types: HashMap<String, usize>,
    pub zoom_levels: HashMap<u32, usize>,
    pub header_values: HashMap<String, HashMap<String, usize>>,
}

impl Summary {
    pub fn to_yaml(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("tiles: {}\n", self.tiles));
        out.push_str(&format!(
            "mean_response_time_ms: {}\n",
            self.mean_response_time_ms
        ));
        out.push_str(&format!(
            "mean_content_length_kb: {}\n",
            self.mean_content_length_kb
        ));

        out.push_str("status_codes:\n");
        write_map(&mut out, &self.status_codes, 2, |k| quote_string(k));
        out.push_str("content_types:\n");
        write_map(&mut out, &self.content_types, 2, |k| quote_string(k));
        out.push_str("zoom_levels:\n");
        write_map(&mut out, &self.zoom_levels, 2, |k| k.to_string());
        out.push_str("headers:\n");
        write_nested_map(&mut out, &self.header_values);
        out
    }
}

fn quote_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn write_map<K, V, F>(out: &mut String, map: &HashMap<K, V>, indent: usize, key_fmt: F)
where
    K: Ord,
    V: Display,
    F: Fn(&K) -> String,
{
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    for (k, v) in entries {
        out.push_str(&" ".repeat(indent));
        out.push_str(&key_fmt(k));
        out.push_str(&format!(": {v}\n"));
    }
}

fn write_nested_map(out: &mut String, map: &HashMap<String, HashMap<String, usize>>) {
    let mut names: Vec<_> = map.iter().collect();
    names.sort_by(|a, b| a.0.cmp(b.0));
    for (name, values) in names {
        out.push_str("  ");
        out.push_str(&quote_string(name));
        out.push_str(":\n");

        let mut entries: Vec<_> = values.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (value, count) in entries {
            out.push_str("    ");
            out.push_str(&quote_string(value));
            out.push_str(&format!(": {count}\n"));
        }
    }
}

/// handles the incoming request metrics and calculates stats.
pub async fn stats_actor(
    mut rx: mpsc::Receiver<StatsEvent>,
    mut state: TuiState,
    tx_ui: mpsc::Sender<TuiState>,
) -> Summary {
    let mut count = 0;

    let _ = tx_ui.send(state.clone()).await;

    // Loop: Receive messages on a channel and update the state
    while let Some(event) = rx.recv().await {
        match event {
            StatsEvent::Url(url) => {
                state.urls.push(url);
                if state.urls.len() > MAX_URLS {
                    state.urls.drain(..state.urls.len() - MAX_URLS);
                }
            }
            StatsEvent::Metric(s) => {
                count += 1;
                state
                    .response_times
                    .push((s.duration.as_secs_f64() * 1000.).round());
                state
                    .response_sizes
                    .push((s.content_length as f64 / 1000.).round());
                state.status_codes = incr_count(state.status_codes, s.status.to_string());
                state.content_types = incr_count(state.content_types, s.content_type.to_string());
                state.zoom_levels = incr_count_u32(state.zoom_levels, s.zoom);
                for (name, value) in s.header_values {
                    let counts = state.header_values.entry(name).or_default();
                    *counts = incr_count(std::mem::take(counts), value);
                }
            }
        }

        let _ = tx_ui.send(state.clone()).await;
    }

    // No more incoming requests, someone dropped the tx end.
    // Mark the UI as complete and emit the final frame.
    state.done = true;
    let _ = tx_ui.send(state.clone()).await;

    Summary {
        tiles: count,
        mean_response_time_ms: mean(&state.response_times),
        mean_content_length_kb: mean(&state.response_sizes),
        status_codes: state.status_codes,
        content_types: state.content_types,
        zoom_levels: state.zoom_levels,
        header_values: state.header_values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn metric() -> RequestMetric {
        RequestMetric::new(
            "http://example.com/3/1/2.pbf".to_string(),
            reqwest::StatusCode::OK,
            1000,
            Duration::from_millis(10),
            Some(&reqwest::header::HeaderValue::from_static(
                "application/x-protobuf",
            )),
            3,
            HashMap::new(),
        )
    }

    #[tokio::test]
    async fn stats_actor_returns_summary_and_marks_done() {
        let (tx, rx) = mpsc::channel(16);
        let (tx_ui, mut rx_ui) = mpsc::channel(16);
        let handle = tokio::spawn(stats_actor(rx, TuiState::default(), tx_ui));

        tx.send(StatsEvent::Metric(metric())).await.unwrap();
        drop(tx);

        let summary = handle.await.unwrap();
        assert_eq!(summary.tiles, 1);
        assert!((summary.mean_response_time_ms - 10.0).abs() < 1e-9);
        assert!((summary.mean_content_length_kb - 1.0).abs() < 1e-9);

        let mut last = TuiState::default();
        while let Ok(state) = rx_ui.try_recv() {
            last = state;
        }
        assert!(last.done);
    }

    #[tokio::test]
    async fn stats_actor_records_urls() {
        let (tx, rx) = mpsc::channel(16);
        let (tx_ui, mut rx_ui) = mpsc::channel(16);
        let handle = tokio::spawn(stats_actor(rx, TuiState::default(), tx_ui));

        tx.send(StatsEvent::Url("http://example.com/1/2/3.pbf".to_string()))
            .await
            .unwrap();
        tx.send(StatsEvent::Metric(metric())).await.unwrap();
        drop(tx);

        let _summary = handle.await.unwrap();

        let mut last = TuiState::default();
        while let Ok(state) = rx_ui.try_recv() {
            last = state;
        }
        assert_eq!(last.urls, vec!["http://example.com/1/2/3.pbf".to_string()]);
    }

    #[tokio::test]
    async fn stats_actor_caps_url_history() {
        let (tx, rx) = mpsc::channel(MAX_URLS + 1);
        let (tx_ui, mut rx_ui) = mpsc::channel(MAX_URLS + 3);
        let handle = tokio::spawn(stats_actor(rx, TuiState::default(), tx_ui));

        for i in 0..(MAX_URLS + 1) {
            tx.send(StatsEvent::Url(format!("url-{i}"))).await.unwrap();
        }
        drop(tx);
        handle.await.unwrap();

        let mut last = TuiState::default();
        while let Ok(state) = rx_ui.try_recv() {
            last = state;
        }
        assert_eq!(last.urls.len(), MAX_URLS);
        assert_eq!(last.urls[0], "url-1");
        assert_eq!(last.urls[MAX_URLS - 1], format!("url-{MAX_URLS}"));
    }

    #[tokio::test]
    async fn stats_actor_counts_header_values() {
        let (tx, rx) = mpsc::channel(16);
        let (tx_ui, _rx_ui) = mpsc::channel(16);
        let handle = tokio::spawn(stats_actor(rx, TuiState::default(), tx_ui));

        let mut m1 = metric();
        m1.header_values
            .insert("X-Cache".to_string(), "HIT".to_string());
        tx.send(StatsEvent::Metric(m1)).await.unwrap();

        let mut m2 = metric();
        m2.header_values
            .insert("X-Cache".to_string(), "MISS".to_string());
        tx.send(StatsEvent::Metric(m2)).await.unwrap();

        let mut m3 = metric();
        m3.header_values
            .insert("X-Cache".to_string(), "HIT".to_string());
        tx.send(StatsEvent::Metric(m3)).await.unwrap();

        drop(tx);
        let summary = handle.await.unwrap();
        let counts = &summary.header_values["X-Cache"];
        assert_eq!(counts.get("HIT"), Some(&2));
        assert_eq!(counts.get("MISS"), Some(&1));
    }

    #[test]
    fn summary_to_yaml_quotes_strings_and_uses_integer_zoom_levels() {
        let summary = Summary {
            tiles: 9,
            mean_response_time_ms: 113.5,
            mean_content_length_kb: 0.5,
            status_codes: HashMap::from([("200 OK".to_string(), 2)]),
            content_types: HashMap::from([("application/x-protobuf".to_string(), 3)]),
            zoom_levels: HashMap::from([(14, 4)]),
            header_values: HashMap::new(),
        };

        assert_eq!(
            summary.to_yaml(),
            "tiles: 9\nmean_response_time_ms: 113.5\nmean_content_length_kb: 0.5\nstatus_codes:\n  \"200 OK\": 2\ncontent_types:\n  \"application/x-protobuf\": 3\nzoom_levels:\n  14: 4\nheaders:\n"
        );
    }

    #[test]
    fn summary_to_yaml_includes_header_value_counts() {
        let summary = Summary {
            tiles: 1,
            mean_response_time_ms: 10.0,
            mean_content_length_kb: 1.0,
            status_codes: HashMap::new(),
            content_types: HashMap::new(),
            zoom_levels: HashMap::new(),
            header_values: HashMap::from([(
                "X-Cache".to_string(),
                HashMap::from([("HIT".to_string(), 2), ("MISS".to_string(), 1)]),
            )]),
        };

        let yaml = summary.to_yaml();
        assert!(yaml.contains("headers:\n  \"X-Cache\":\n    \"HIT\": 2\n    \"MISS\": 1\n"));
    }
}
