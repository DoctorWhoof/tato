use tato_layout::Rect;

use crate::*;

impl Tato {
    pub fn draw_patch(&mut self, map: u8, rect: Rect<u16>, tileset_id: TilesetID) {
        let tileset = &self.tiles.sets[tileset_id.0 as usize];
        assert!(tileset.len == 9, err!("Tile patch tilesets must be 9 tiles long."));

        let map = &mut self.maps[map as usize];
        let tile_index = TileID(tileset.start);

        map.set_tile(BgBundle {
            col: rect.x,
            row: rect.y,
            tile_id: tile_index,
            flags: TileFlags::default(),
        });
    }

    /// Copies a rectangular region from a source tilemap to this tilemap.
    /// - If `src_rect` is None, attempts to copy the entire source tilemap.
    /// - If `dst_rect` is None, pastes at (0,0) and fills as many tiles as possible.
    /// - Negative destination coordinates are handled by clipping the source region.
    pub fn copy_tile_rect<const S_LEN: usize, const D_LEN: usize>(
        source: &Tilemap<S_LEN>,
        dest: &mut Tilemap<D_LEN>,
        src_rect: Option<&Rect<u16>>,
        dst_rect: Option<&Rect<u16>>,
    ) {
        // Determine source rectangle
        let src_x = src_rect.map_or(0, |r| r.x) as i16;
        let src_y = src_rect.map_or(0, |r| r.y) as i16;
        let src_w = src_rect.map_or(source.columns, |r| r.w) as i16;
        let src_h = src_rect.map_or(source.rows(), |r| r.h) as i16;

        // Make sure source rectangle is within bounds
        let src_w = i16::min(src_w, source.columns as i16 - src_x);
        let src_h = i16::min(src_h, source.rows() as i16 - src_y);

        // Determine destination rectangle
        let dst_x = dst_rect.map_or(0, |r| r.x) as i16;
        let dst_y = dst_rect.map_or(0, |r| r.y) as i16;

        // Calculate clipping for negative coordinates
        let clip_x = if dst_x < 0 { -dst_x } else { 0 };
        let clip_y = if dst_y < 0 { -dst_y } else { 0 };

        // Adjust source and destination starting points
        let effective_src_x = src_x + clip_x;
        let effective_src_y = src_y + clip_y;
        let effective_dst_x = i16::max(0, dst_x);
        let effective_dst_y = i16::max(0, dst_y);

        // Calculate effective width and height after clipping
        let effective_width = i16::max(0, i16::min(src_w - clip_x, dest.columns as i16 - effective_dst_x));
        let effective_height = i16::max(0, i16::min(src_h - clip_y, dest.rows() as i16 - effective_dst_y));

        // If there's nothing to copy (zero width or height), return early
        if effective_width <= 0 || effective_height <= 0 {
            return;
        }

        // Copy the tiles row by row
        for y in 0..effective_height {
            for x in 0..effective_width {
                let src_index = (effective_src_y + y) as usize * source.columns as usize + (effective_src_x + x) as usize;
                let dst_index = (effective_dst_y + y) as usize * dest.columns as usize + (effective_dst_x + x) as usize;

                // Ensure we're within bounds (additional safety check)
                // if src_index < S_LEN && dst_index < LEN {
                dest.data[dst_index] = source.data[src_index];
                // }
            }
        }
    }
}
