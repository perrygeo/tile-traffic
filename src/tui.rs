//! Terminal UI for tile-traffic, rendered with ratatui.
//!
use std::collections::HashMap;
use std::fmt::Display;
use std::io::IsTerminal;

use anyhow::Result;
use cli_hist::bucketers::linear_bucketer::LinearBucketer;
use cli_hist::histogram::Histogram;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use tokio::sync::mpsc;

const HIST_BINS: u8 = 8;
const MAX_COUNT_LINES: usize = 9;
const URL_VIEW_LINES: usize = 6;

#[derive(Clone, Default)]
pub struct TuiState {
    pub response_times: Vec<f64>,
    pub response_sizes: Vec<f64>,
    pub status_codes: HashMap<String, usize>,
    pub content_types: HashMap<String, usize>,
    pub zoom_levels: HashMap<u32, usize>,
    pub header_values: HashMap<String, HashMap<String, usize>>,
    pub urls: Vec<String>,
    pub done: bool,
}

fn histogram_text(values: &[f64], bins: u8) -> String {
    let mut histogram = Histogram::new('█');
    for value in values {
        histogram.insert(value.round() as u32);
    }
    format!("{}", histogram.bucket(&LinearBucketer::new(bins)))
}

fn count_text<K: Ord + Display>(title: &str, counts: &HashMap<K, usize>) -> Text<'static> {
    let mut lines = vec![Line::from(Span::styled(
        title.to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    if counts.is_empty() {
        lines.push(Line::from("(none)"));
    } else {
        let mut entries: Vec<_> = counts.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let fits = entries.len() < MAX_COUNT_LINES;
        let shown = if fits {
            entries.len()
        } else {
            MAX_COUNT_LINES - 2
        };
        for (key, value) in entries.into_iter().take(shown) {
            lines.push(Line::from(format!("{key}: {value}")));
        }
        if !fits {
            lines.push(Line::from("..."));
        }
    }
    Text::from(lines)
}

fn count_height(state: &TuiState) -> u16 {
    let mut entries = state
        .status_codes
        .len()
        .max(state.content_types.len())
        .max(state.zoom_levels.len());
    for values in state.header_values.values() {
        entries = entries.max(values.len());
    }
    (entries.clamp(1, MAX_COUNT_LINES - 1) + 1) as u16
}

fn url_text(urls: &[String]) -> Text<'static> {
    let mut lines = vec![Line::from(Span::styled(
        "Recent URLs".to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    if urls.is_empty() {
        lines.push(Line::from("(none)"));
    } else {
        let view = URL_VIEW_LINES - 1;
        let start = urls.len().saturating_sub(view);
        for url in &urls[start..] {
            lines.push(Line::from(url.clone()));
        }
    }
    Text::from(lines)
}

pub fn render(frame: &mut Frame, state: &TuiState) {
    let [header, histograms, urls, counts, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(URL_VIEW_LINES as u16),
        Constraint::Length(count_height(state)),
        Constraint::Length(if state.done { 1 } else { 0 }),
    ])
    .areas(frame.area());

    let tiles = state.response_times.len();
    frame.render_widget(
        Paragraph::new(format!("tile-traffic · {tiles} tiles"))
            .style(Style::default().add_modifier(Modifier::BOLD)),
        header,
    );

    let [times_area, sizes_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(histograms);

    frame.render_widget(
        Paragraph::new(histogram_text(&state.response_times, HIST_BINS))
            .block(Block::bordered().title("Response times (ms)")),
        times_area,
    );
    frame.render_widget(
        Paragraph::new(histogram_text(&state.response_sizes, HIST_BINS))
            .block(Block::bordered().title("Content lengths (kB)")),
        sizes_area,
    );

    frame.render_widget(Paragraph::new(url_text(&state.urls)), urls);

    let mut header_names: Vec<&String> = state.header_values.keys().collect();
    header_names.sort();

    let column_count = 3 + header_names.len();
    let areas = Layout::horizontal(vec![
        Constraint::Ratio(1, column_count as u32);
        column_count
    ])
    .split(counts);

    frame.render_widget(
        Paragraph::new(count_text("Status codes", &state.status_codes)),
        areas[0],
    );
    frame.render_widget(
        Paragraph::new(count_text("Content types", &state.content_types)),
        areas[1],
    );
    frame.render_widget(
        Paragraph::new(count_text("Zoom levels", &state.zoom_levels)),
        areas[2],
    );
    for (i, name) in header_names.into_iter().enumerate() {
        frame.render_widget(
            Paragraph::new(count_text(name.as_str(), &state.header_values[name])),
            areas[3 + i],
        );
    }

    if state.done {
        frame.render_widget(
            Paragraph::new("press any key to exit")
                .style(Style::default().add_modifier(Modifier::BOLD)),
            footer,
        );
    }
}

pub async fn tui_actor(mut rx: mpsc::Receiver<TuiState>) -> Result<()> {
    if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
        return Ok(());
    }
    let mut terminal = ratatui::try_init()?;
    while let Some(state) = rx.recv().await {
        terminal.draw(|frame| render(frame, &state))?;
    }
    wait_for_key()?;
    ratatui::try_restore()?;
    Ok(())
}

fn wait_for_key() -> Result<()> {
    use ratatui::crossterm::event::{self, Event, KeyEventKind};
    loop {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => return Ok(()),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::Terminal;

    fn buffer_text(buffer: &Buffer) -> String {
        let area = *buffer.area();
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_to_string(state: &TuiState) -> String {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, state)).unwrap();
        buffer_text(terminal.backend().buffer())
    }

    #[test]
    fn histogram_text_is_empty_when_no_values() {
        assert_eq!(histogram_text(&[], 8), "Empty histogram\n");
    }

    #[test]
    fn histogram_text_renders_buckets_and_counts() {
        let text = histogram_text(&[1.0, 2.0, 4.0], 2);
        assert!(text.contains('█'), "expected histogram bars, got: {text}");
        assert!(
            text.contains("(2)"),
            "expected a bucket count of 2, got: {text}"
        );
        assert!(
            text.contains("(1)"),
            "expected a bucket count of 1, got: {text}"
        );
    }

    #[test]
    fn render_shows_headers_and_counts() {
        let mut state = TuiState::default();
        state.response_times.push(10.0);
        state.response_sizes.push(5.0);
        state.status_codes.insert("200 OK".to_string(), 1);
        state
            .content_types
            .insert("application/x-protobuf".to_string(), 1);
        state.zoom_levels.insert(4, 1);

        let text = render_to_string(&state);
        assert!(text.contains("tile-traffic"), "missing header: {text}");
        assert!(
            text.contains("Response times (ms)"),
            "missing times title: {text}"
        );
        assert!(
            text.contains("Content lengths (kB)"),
            "missing sizes title: {text}"
        );
        assert!(
            text.contains("Status codes"),
            "missing status title: {text}"
        );
        assert!(text.contains("200 OK: 1"), "missing status count: {text}");
        assert!(
            text.contains("application/x-protobuf: 1"),
            "missing content type count: {text}"
        );
        assert!(text.contains("Zoom levels"), "missing zoom title: {text}");
    }

    #[test]
    fn render_handles_small_terminal() {
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &TuiState::default()))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("tile-traffic"), "missing header: {text}");
    }

    #[test]
    fn render_shows_url_section_when_empty() {
        let text = render_to_string(&TuiState::default());
        assert!(text.contains("Recent URLs"), "missing urls title: {text}");
        assert!(text.contains("(none)"), "missing empty hint: {text}");
    }

    #[test]
    fn render_shows_recent_urls_and_scrolls_off_oldest() {
        let state = TuiState {
            urls: (0..8)
                .map(|i| format!("https://tiles.example.com/{i}/1/2.pbf"))
                .collect(),
            ..TuiState::default()
        };

        let text = render_to_string(&state);
        assert!(text.contains("Recent URLs"), "missing urls title: {text}");
        assert!(
            text.contains("https://tiles.example.com/7/1/2.pbf"),
            "missing newest url: {text}"
        );
        assert!(
            !text.contains("https://tiles.example.com/0/1/2.pbf"),
            "oldest url should scroll off: {text}"
        );
    }

    #[test]
    fn render_shows_done_hint_when_finished() {
        let state = TuiState {
            done: true,
            ..TuiState::default()
        };
        let text = render_to_string(&state);
        assert!(
            text.contains("press any key to exit"),
            "missing done hint: {text}"
        );
    }

    #[test]
    fn render_shows_header_value_counts() {
        let mut state = TuiState::default();
        state.header_values.insert(
            "X-Cache".to_string(),
            HashMap::from([("HIT".to_string(), 2), ("MISS".to_string(), 1)]),
        );

        let text = render_to_string(&state);
        assert!(text.contains("X-Cache"), "missing header name: {text}");
        assert!(text.contains("HIT: 2"), "missing HIT count: {text}");
        assert!(text.contains("MISS: 1"), "missing MISS count: {text}");
    }

    #[test]
    fn render_shows_all_eight_header_values_without_ellipsis() {
        let mut state = TuiState::default();
        let counts: HashMap<String, usize> = (0..8).map(|i| (format!("v{i:02}"), 1)).collect();
        state.header_values.insert("X-High".to_string(), counts);

        let text = render_to_string(&state);
        assert!(!text.contains("..."), "unexpected ellipsis: {text}");
        assert!(text.contains("v07: 1"), "missing last value: {text}");
    }

    #[test]
    fn render_truncates_nine_header_values_with_ellipsis() {
        let mut state = TuiState::default();
        let counts: HashMap<String, usize> = (0..9).map(|i| (format!("v{i:02}"), 1)).collect();
        state.header_values.insert("X-High".to_string(), counts);

        let text = render_to_string(&state);
        assert!(text.contains("..."), "missing ellipsis: {text}");
        assert!(text.contains("v06: 1"), "missing last shown value: {text}");
        assert!(
            !text.contains("v07: 1"),
            "value past cap should be hidden: {text}"
        );
    }
}
