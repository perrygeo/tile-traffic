use clap::Parser;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use tile_traffic::coordinates::Tile;
use tile_traffic::request_handler::request_handler;
use tile_traffic::statistics::stats_actor;
use tile_traffic::strategies::Metatile;

#[derive(Parser, Debug)]
struct Args {
    template: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let n_bursts = 20;
    let n_requests_per_burst = 16;
    let buffer = 32;

    // Spawn actor to handle the stats and terminal drawing
    let (tx_stats, rx_stats) = mpsc::channel(buffer);
    // TODO create the tuistate and move it
    let stats_handle = tokio::spawn(async move { stats_actor(rx_stats).await });

    // Specify the strategy for this session
    let starting_tile = Tile::from_coords(-104.99, 39.72, 6);
    let strategy = Metatile::new(args.template, starting_tile, 10);

    for b in 0..n_bursts {
        // Collect all the futures for this "burst"
        let mut tasks = (0..n_requests_per_burst)
            .into_iter()
            .map(|r| {
                let strat = strategy.clone();
                let seed = (b * n_requests_per_burst) + r;
                let tx = tx_stats.clone();
                request_handler(strat, seed, tx)
            })
            .collect::<FuturesUnordered<_>>();

        // Wait for all tile requests to complete
        while tasks.next().await.is_some() {}

        // Sleep for bit; when the user pauses to "view" the map
        sleep(Duration::from_millis(100)).await;
    }

    // clean up channels to ensure completion of tasks
    drop(tx_stats);
    stats_handle.await?;

    Ok(())
}
