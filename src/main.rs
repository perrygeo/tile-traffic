use anyhow::{bail, Result};
use clap::Parser;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use tile_traffic::coordinates::Tile;
use tile_traffic::request_handler::request_handler;
use tile_traffic::statistics::stats_actor;
use tile_traffic::strategies::{Metatile, Template};
use tile_traffic::tui::TuiState;

const BUFFER: usize = 32;

#[derive(Parser, Debug)]
struct Args {
    /// Number of bursts
    #[clap(short, long, default_value_t = 16)]
    bursts: usize,

    /// Number of requests per burst
    #[clap(short, long, default_value_t = 16)]
    requests_per_burst: usize,

    /// WMS template
    #[clap(long)]
    wms: Option<String>,

    /// ZXY template
    #[clap(long)]
    zxy: Option<String>,

    /// Latitude
    #[clap(long, allow_hyphen_values(true))]
    lat: f64,

    /// Longitude
    #[clap(long, allow_hyphen_values(true))]
    lon: f64,

    /// Starting zoom level (higher - zoomed in)
    #[clap(long)]
    start_zoom: u32,

    /// End at zoom level (lower - zoomed out)
    #[clap(long)]
    end_zoom: u32,

    /// Sleep for a while between bursts, ms
    #[clap(long, default_value_t = 10)]
    sleep_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let args = Args::parse();

    // TODO Validate/Parse template into an impl TileTemplate
    if args.zxy.is_some() && args.wms.is_some() {
        bail!("--zxy and --wms are mututally exclusive");
    }
    let (template, template_type) = if let Some(t) = args.zxy {
        (t, Template::Zxy)
    } else if let Some(t) = args.wms {
        (t, Template::Wms)
    } else {
        bail!("One of --zxy <template> or --wms <template> is required");
    };

    // Specify the strategy for this session
    let tile = Tile::from_coords(args.lon, args.lat, args.end_zoom);
    let strategy = Metatile::new(tile, args.start_zoom, template, template_type);

    // Create state and a channel to update it
    let state = TuiState::default();
    let (tx_stats, rx_stats) = mpsc::channel(BUFFER);

    // Spawn actor to handle the request statistics and update state
    let stats_handle = tokio::spawn(async move { stats_actor(rx_stats, state).await });

    for b in 0..args.bursts {
        // Collect all the futures for this "burst"
        let mut tasks = (0..args.requests_per_burst)
            .into_iter()
            .map(|r| {
                let strat = strategy.clone();
                let seed = (b * args.requests_per_burst) + r;
                let tx = tx_stats.clone();
                request_handler(strat, seed, tx)
            })
            .collect::<FuturesUnordered<_>>();

        // Wait for all tile requests to complete
        while tasks.next().await.is_some() {}

        // Sleep for bit; when the user pauses to "view" the map
        sleep(Duration::from_millis(args.sleep_ms)).await;
    }

    // clean up channels to ensure completion of tasks
    drop(tx_stats);
    stats_handle.await?;

    Ok(())
}
