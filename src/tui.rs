//! Draw to the terminal
//!
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins};
use plotlib::view::ContinuousView;
use std::collections::HashMap;

pub struct TuiState<'a> {
    pub response_times: &'a Vec<f64>,
    pub response_sizes: &'a Vec<f64>,
    pub status_codes: &'a HashMap<String, usize>,
    pub content_types: &'a HashMap<String, usize>,
}

pub fn draw(state: TuiState) {
    let times = state.response_times;
    let sizes = state.response_sizes;
    let bins = 40;

    let h = Histogram::from_slice(times, HistogramBins::Count(bins));
    let v = ContinuousView::new().add(h);
    let response_time_hist = Page::single(&v).dimensions(82, 9).to_text().unwrap();

    let h = Histogram::from_slice(sizes, HistogramBins::Count(bins));
    let v = ContinuousView::new().add(h);
    let response_size_hist = Page::single(&v).dimensions(82, 9).to_text().unwrap();

    // clear screen and redraw
    print!("{esc}c", esc = 27 as char);
    println!(
        "Running tile-traffic...
Completed {} requests
                                   Response times (ms)
{}
                                   Response sizes (kB)
{}

Status Codes
{:#?}

Content Types
{:#?}
        ",
        times.len(),
        response_time_hist,
        response_size_hist,
        state.status_codes,
        state.content_types
    );
}
