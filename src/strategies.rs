//! Simulate web mapping user sessions
//!
use crate::coordinates::Tile;

pub trait WebMapSession {
    fn make_url(&self, seed: usize) -> String;
}

#[derive(Clone, Debug)]
pub struct Metatile {
    template: String,
    metatile: Tile,
    to_zoom: u32,
}

impl Metatile {
    pub fn new(template: String, metatile: Tile, to_zoom: u32) -> Self {
        Metatile {
            template,
            metatile,
            to_zoom,
        }
    }
}

impl WebMapSession for Metatile {
    fn make_url(&self, seed: usize) -> String {
        // TODO we can cache this
        let children = self.metatile.children(self.to_zoom);

        let idx = seed % children.len();
        let tile = &children[idx];

        let mut url = self.template.clone();
        url = url.replace("{x}", tile.x.to_string().as_ref());
        url = url.replace("{y}", tile.y.to_string().as_ref());
        url = url.replace("{z}", tile.zoom.to_string().as_ref());
        url
    }
}
