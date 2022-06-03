//! Draw to the terminal
//!
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins};
use plotlib::view::ContinuousView;
use std::collections::HashMap;

pub struct TuiState {
    pub response_times: Vec<f64>,
    pub response_sizes: Vec<f64>,
    pub status_codes: HashMap<String, usize>,
    pub content_types: HashMap<String, usize>,
    pub zoom_levels: HashMap<String, usize>,
}

impl TuiState {
    pub fn default() -> Self {
        TuiState {
            response_times: Vec::new(),
            response_sizes: Vec::new(),
            status_codes: HashMap::new(),
            content_types: HashMap::new(),
            zoom_levels: HashMap::new(),
        }
    }
    pub fn draw(&self) {
        let times = &self.response_times;
        let sizes = &self.response_sizes;
        let bins = 36;

        let h = Histogram::from_slice(times, HistogramBins::Count(bins));
        let v = ContinuousView::new().add(h);
        let response_time_hist = Page::single(&v).dimensions(36, 7).to_text().unwrap();

        let h = Histogram::from_slice(sizes, HistogramBins::Count(bins));
        let v = ContinuousView::new().add(h);
        let response_size_hist = Page::single(&v).dimensions(36, 7).to_text().unwrap();

        // clear screen and redraw
        print!("{esc}c", esc = 27 as char);
        println!(
            "Running tile-traffic...

              Response times (ms)
{}

              Content Lengths (kB)
{}

Count by Status Code
{:?}

Count by Content Type
{:?}

Count by Zoom Level
{:?}
            ",
            response_time_hist,
            response_size_hist,
            self.status_codes,
            self.content_types,
            self.zoom_levels
        );
    }
}
