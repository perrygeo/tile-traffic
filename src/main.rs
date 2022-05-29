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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    if let Some(template) = args.zxy {
        let futures = FuturesUnordered::new();
        for c in 0..24 {
            let mut url = template.clone();
            url = url.replace("{z}", ((c % 4) + 4).to_string().as_ref());
            url = url.replace("{x}", ((c * 4) + 4).to_string().as_ref());
            url = url.replace("{y}", ((c * 4) + 4).to_string().as_ref());
            futures.push(tokio::spawn(async move {
                let start = Instant::now();
                let res = reqwest::get(url).await;
                let duration = start.elapsed();
                info!("{:?}", duration);
                res
            }));
        }

        futures
            .for_each_concurrent(4, |r| async move {
                if let Ok(Ok(response)) = r {
                    let status = response.status();
                    let path = response.url().path();
                    let content_length = response.content_length().unwrap_or(0);
                    // let headers = response.headers();
                    // let content_length_header = headers.get("Content-Length");
                    info!("{} {:?}, {:?}", path, status, content_length);
                } else {
                    error!("{:?}", r);
                }
            })
            .await;
    }

    if args.wms.is_some() {
        unimplemented!();
    }

    Ok(())
}
