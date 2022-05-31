#[derive(Clone)]
pub enum Strategy {
    Metatile(String),
    FlightSim(String),
}

#[derive(Clone, Debug)]
struct Tile {
    x: u32,
    y: u32,
    zoom: u32,
}

impl Tile {
    fn children(&self, target_zoom: u32) -> Vec<Tile> {
        let metatile = Tile {
            x: self.x,
            y: self.y,
            zoom: self.zoom,
        };
        let mut tiles = vec![metatile];
        for z in self.zoom..=target_zoom {
            let prev_zoom = z - 1;

            // this is a hack algorithm!
            // TODO eliminate clone and
            // only iterate over tiles of the previous zoom
            for t in tiles.clone().iter() {
                if t.zoom == prev_zoom {
                    tiles.push(Tile {
                        x: t.x * 2,
                        y: t.y * 2,
                        zoom: z,
                    });
                    tiles.push(Tile {
                        x: t.x * 2 + 1,
                        y: t.y * 2,
                        zoom: z,
                    });
                    tiles.push(Tile {
                        x: t.x * 2 + 1,
                        y: t.y * 2 + 1,
                        zoom: z,
                    });
                    tiles.push(Tile {
                        x: t.x * 2,
                        y: t.y * 2 + 1,
                        zoom: z,
                    });
                }
            }
        }

        tiles
    }
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
