use clap::Parser;
use tokio::sync::mpsc;

use webmap_loadgen::request_handler::request_handler;
use webmap_loadgen::statistics::stats_actor;
use webmap_loadgen::strategies::Strategy;

#[derive(Parser, Debug)]
struct Args {
    template: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let n_bursts = 8;
    let n_requests_per_burst = 32;
    let buffer = 32;

    // Spawn actor to handle the stats and terminal drawing
    let (tx_stats, rx_stats) = mpsc::channel(buffer);
    let stats_handle = tokio::spawn(async move { stats_actor(rx_stats).await });

    // Specify the strategy for this session
    let strategy = Strategy::Metatile(args.template);

    for b in 0..n_bursts {
        let mut join_handles = Vec::new();
        for rpb in 1..=n_requests_per_burst {
            // Spawn a request
            let strat = strategy.clone();
            let seed = (b * n_requests_per_burst) + rpb;
            let tx = tx_stats.clone();
            join_handles.push(tokio::spawn(async move {
                request_handler(strat, seed, tx).await
            }));
        }
        // Await all requests to ensure completion
        for jh in join_handles {
            jh.await?;
        }
    }

    // Note: Must clean up channels and ensure completion of tasks
    drop(tx_stats);
    stats_handle.await?;

    Ok(())
}
