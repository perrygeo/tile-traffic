use std::f64::{consts::E, EPSILON};

#[derive(Clone, Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}

impl Tile {
    pub fn from_coords(lon: f64, lat: f64, zoom: u32) -> Self {
        let x = lon.to_radians() / 360. + 0.5;

        let sinlat = lat.to_radians().sin();
        let y = 0.5 - 0.25 * ((1. + sinlat) / (1. - sinlat)).log(E) / std::f64::consts::PI;

        let z2 = zoom.pow(2);

        let xtile = if x <= 0. {
            0
        } else if x >= 1. {
            z2 - 1
        } else {
            ((x + EPSILON) * z2 as f64) as u32
        };

        let ytile = if y <= 0. {
            0
        } else if x >= 1. {
            z2 - 1
        } else {
            ((y + EPSILON) * z2 as f64) as u32
        };

        Tile {
            x: xtile,
            y: ytile,
            zoom,
        }
    }

    pub fn children(&self, target_zoom: u32) -> Vec<Self> {
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

mod test {
    #[test]
    fn test_tile() {
        let t = super::Tile::from_coords(-120., 44., 7);
        assert_eq!(t.zoom, 7);
        assert_eq!(t.x, 24);
        assert_eq!(t.y, 17);
    }
}
