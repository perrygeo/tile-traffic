use crate::coordinates::Tile;

#[derive(Clone)]
pub enum Strategy {
    Metatile(String),
    FlightSim(String),
}

impl Strategy {
    pub fn make_url(&self, c: usize) -> String {
        let url = match self {
            Strategy::Metatile(url) => {
                let mut url = url.clone();

                let metatile = Tile {
                    x: 26,
                    y: 48,
                    zoom: 7,
                };
                let children = metatile.children(9);
                let idx = (c + 1) % children.len();
                let tile = &children[idx];

                // http://localhost:7800/osm.points/z/x/y.pbf
                url = url.replace("{x}", tile.x.to_string().as_ref());
                url = url.replace("{y}", tile.y.to_string().as_ref());
                url = url.replace("{z}", tile.zoom.to_string().as_ref());
                url
            }
            Strategy::FlightSim(_) => todo!(),
        };
        url
    }
}
