use clap::Parser;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use log::{error, info};
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    zxy: Option<String>,

    #[clap(short, long)]
    wms: Option<String>,
}

// configuration.rs
// coordinates.rs list of pts to zxys or lonlat extents
// lib.rs
// services/wms.rs
// services/zxy.rs
// statistics/analysis.rs reads from sqlite
// statistics/collector.rs writes to sqlite
// strategies/orbit.rs
// strategies/search_and_pan.rs
// tui.rs
// map_browsing_session.rs
// workers.rs

// map_browsing_session.rs combines service + strategy + config + coords
// e.g. localhost tileserver + orbit + all 60 utm zones + starting at dallas

pub async fn worker() {
    // TODO
    todo!();
}

pub fn make_url(template: String, c: usize) -> String {
    let mut url = template;
    // http://localhost:7800/osm.points/8/131/93.pbf
    // Move diagonally
    let z = 7;
    let x = 102;
    let y = 53;

    url = url.replace("{z}", z.to_string().as_ref());
    url = url.replace("{x}", x.to_string().as_ref());
    url = url.replace("{y}", y.to_string().as_ref());
    url
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    if let Some(template) = args.zxy {
        let n_bursts = 4;
        let n_requests_per_burst = 4;
        for b in 0..n_bursts {
            let futures = FuturesUnordered::new();
            for rpb in 0..n_requests_per_burst {
                let url = make_url(template.clone(), b * rpb);

                futures.push(async move {
                    info!("iniating request");
                    let start = Instant::now();
                    let res = reqwest::get(url).await;
                    let duration = start.elapsed();
                    if let Ok(response) = res {
                        let status = response.status();
                        let path = response.url().to_string();
                        let content_length = response.content_length().unwrap_or(0);
                        info!(
                            "{} {:?}, {:?}, {:?}",
                            path, status, content_length, duration
                        );
                    } else {
                        error!("{:?}", res);
                    };
                });
            }

            futures.for_each_concurrent(3, |_| async move {}).await;
        }
    }

    if args.wms.is_some() {
        unimplemented!();
    }

    Ok(())
}
