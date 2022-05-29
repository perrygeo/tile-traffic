use clap::Parser;
use tokio::sync::mpsc;

use webmap_loadgen::request_handler::request_handler;
use webmap_loadgen::statistics::stats_actor;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    zxy: Option<String>,

    #[clap(short, long)]
    wms: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let n_bursts = 4;
    let n_requests_per_burst = 16;
    let stats_buffer = 32;

    let (tx_stats, rx_stats) = mpsc::channel(stats_buffer);
    let stats_handle = tokio::spawn(async move { stats_actor(rx_stats).await });

    if let Some(template) = args.zxy {
        for b in 0..n_bursts {
            let mut join_handles = Vec::new();
            for rpb in 1..=n_requests_per_burst {
                let tmpl = template.clone();
                let seed = (b * n_requests_per_burst) + rpb;
                let tx = tx_stats.clone();
                // // TODO config should tell us how to construct these...
                // let session = Session::FlightSim::new(XYZ Template, StartCoord, EndCoord)
                // let session = Session::Metatile::new(XYZ Template, StartingTile, EndZoom)
                // pass a `MapBrowsingSession` + seed to request_handler
                join_handles.push(tokio::spawn(async move {
                    request_handler(tmpl, seed, tx).await
                }));
            }
            for jh in join_handles {
                jh.await?;
            }
        }
    }

    if args.wms.is_some() {
        unimplemented!();
    }

    // Must clean up stats actor to ensure completion
    drop(tx_stats);
    stats_handle.await?;

    Ok(())
}
