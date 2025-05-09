use crate::*;

#[derive(Debug, Clone)]
pub struct TileBank {
    pub tiles: [Tile<2>; TILE_COUNT],
    tile_id_head: usize,
    tile_pixel_head: usize,
}

impl Default for TileBank {
    fn default() -> Self {
        Self {
            tiles: core::array::from_fn(|_| Tile::default()),
            tile_id_head: 0,
            tile_pixel_head: 0,
        }
    }
}

impl TileBank {
    pub fn reset(&mut self) {
        self.tile_id_head = 0;
        self.tile_pixel_head = 0;
    }

    pub fn new_tile(&mut self, data: &[u8]) -> TileID {
        // Check if number of pixels is correct
        assert!(
            data.len() == TILE_PIXEL_COUNT,
            err!("Tile data length must match TILE_PIXEL_COUNT ({})"),
            TILE_PIXEL_COUNT
        );

        // Check if we have enough space
        if self.tile_id_head >= TILE_COUNT {
            panic!(err!("Not enough space for new tile"))
        }

        let tile_id = u16::try_from(self.tile_id_head).unwrap();

        // Pack 8 pixels (2 bits each) into each cluster
        // TODO: REPLACE WITH TILE.SET_PIXEL
        let mut cluster_index = 0;
        let mut subpixel_index = 0;
        for i in 0..TILE_PIXEL_COUNT {
            // Clamp color to maximum allowed
            let value = data[i].clamp(0, COLORS_PER_TILE as u8);

            // Set pixel data
            self.tiles[self.tile_id_head] //
                .clusters[cluster_index]
                .set_subpixel(value, subpixel_index);

            // Advance
            subpixel_index += 1;
            if subpixel_index >= PIXELS_PER_CLUSTER {
                subpixel_index = 0;
                cluster_index += 1;
            }
        }

        self.tile_id_head += 1;
        self.tile_pixel_head += TILE_PIXEL_COUNT;

        TileID(tile_id)
    }
}
