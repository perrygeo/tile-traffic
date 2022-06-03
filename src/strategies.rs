//! Simulate web mapping user sessions
//!
use crate::coordinates::Tile;

pub trait WebMapSession {
    fn make_url(&self, seed: usize) -> (String, Tile);
}

#[derive(Clone, Debug)]
pub struct Metatile {
    metatile: Tile,
    to_zoom: u32,
    template: String,
    template_type: Template,
}

#[derive(Clone, Debug)]
pub enum Template {
    Zxy,
    Wms,
}

impl Metatile {
    pub fn new(metatile: Tile, to_zoom: u32, template: String, template_type: Template) -> Self {
        Metatile {
            metatile,
            to_zoom,
            template,
            template_type,
        }
    }
}

impl WebMapSession for Metatile {
    fn make_url(&self, seed: usize) -> (String, Tile) {
        let children = self.metatile.children(self.to_zoom);
        let idx = seed % children.len();
        let tile = &children[idx];

        let url = match self.template_type {
            Template::Zxy => tile.url_zyx(self.template.clone()),
            Template::Wms => tile.url_wms(self.template.clone()),
        };

        (url, tile.clone())
    }
}
