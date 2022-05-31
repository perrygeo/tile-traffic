use crate::coordinates::Tile;

pub trait WebMapSession {
    fn make_url(&self, seed: usize) -> String;
}

#[derive(Clone, Debug)]
pub struct Metatile {
    template: String,
}

impl Metatile {
    pub fn new(template: String) -> Self {
        Metatile { template }
    }
}

impl WebMapSession for Metatile {
    fn make_url(&self, seed: usize) -> String {
        let mut url = self.template.clone();

        let metatile = Tile {
            x: 26,
            y: 48,
            zoom: 7,
        };
        let children = metatile.children(9);
        let idx = (seed + 1) % children.len();
        let tile = &children[idx];

        // http://localhost:7800/osm.points/z/x/y.pbf
        url = url.replace("{x}", tile.x.to_string().as_ref());
        url = url.replace("{y}", tile.y.to_string().as_ref());
        url = url.replace("{z}", tile.zoom.to_string().as_ref());
        url
    }
}
