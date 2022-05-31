use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins};
use plotlib::view::ContinuousView;

pub struct TuiState<'a> {
    pub response_times: &'a Vec<f64>,
    pub response_sizes: &'a Vec<f64>,
}

pub fn draw(state: TuiState) {
    let times = state.response_times;
    let sizes = state.response_sizes;

    let h = Histogram::from_slice(times, HistogramBins::Count(80));
    let v = ContinuousView::new().add(h);
    let response_time_hist = Page::single(&v).dimensions(82, 7).to_text().unwrap();

    let h = Histogram::from_slice(sizes, HistogramBins::Count(80));
    let v = ContinuousView::new().add(h);
    let response_size_hist = Page::single(&v).dimensions(82, 7).to_text().unwrap();

    // clear screen and redraw
    print!("{esc}c", esc = 27 as char);
    println!(
        "Running tile-traffic...
Completed {} requests
                                   Response times (ms)
{}
                                   Response sizes (kB)
{}
        ",
        times.len(),
        response_time_hist,
        response_size_hist
    );
}
