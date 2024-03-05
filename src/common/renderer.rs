use super::*;
use core::{mem::variant_count, array};

/// Loads and stores fixed size tiles organized into tilesets that can be added and removed individually.
pub struct Renderer<S, TilesetEnum, PaletteEnum>
where
    S:Specs,
    TilesetEnum:Into<u8> + Into<usize> + Copy,
    PaletteEnum:Into<u8> + Into<usize> + Copy,
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::MAX_LOADED_TILESETS]: Sized,
    [(); S::MAX_LOADED_FONTS]: Sized,
    [(); S::MAX_LOADED_ANIMS]: Sized,
    [(); S::MAX_LOADED_TILEMAPS]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,                                                          // Tile Pixels
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,                                // Max Per Tileset pixels
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized,     // Tile count
{
    pub(crate) partitions:  [Option<Partition>; variant_count::<TilesetEnum>()],
    pub(crate) palettes:    [Palette<S>; variant_count::<PaletteEnum>()],
    fonts:                  [Option<Font>; S::MAX_LOADED_FONTS],
    tilemaps:               [Option<Tilemap>; S::MAX_LOADED_TILEMAPS],
    pixels:                 [u8; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
    rects:                  [Rect<u8>; (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)],
    anims:                  [Option<Anim>; S::MAX_LOADED_ANIMS],
    partition_top:          Option<u8>
}


impl< S, TilesetEnum, PaletteEnum> Renderer<S, TilesetEnum, PaletteEnum>
where
    S:Specs,
    TilesetEnum:Into<u8> + Into<usize> + Copy,
    PaletteEnum:Into<u8> + Into<usize> + Copy,
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::MAX_LOADED_TILESETS]: Sized,
    [(); S::MAX_LOADED_FONTS]: Sized,
    [(); S::MAX_LOADED_ANIMS]: Sized,
    [(); S::MAX_LOADED_TILEMAPS]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,  // Pixel count
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized, //Tile count
{
    pub(crate) fn new() -> Self {
        
        #[cfg(feature = "std")]{
            let tile_count = (S::ATLAS_WIDTH * S::ATLAS_HEIGHT) / (S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize);
            println!("Renderer: Creating new Renderer with {} tiles.", tile_count);
        }

        Renderer {
            pixels: [0; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
            // Global Assets are always initialized. The length of the array containing each asset
            // is determined by the Enum associated with it, and the builder script will fail if any
            // enum variant is not initialized
            palettes: core::array::from_fn( |i| Palette::new( u8::try_from(i).unwrap() )),
            partitions: core::array::from_fn( |_| None ),
            partition_top: None,
            fonts: core::array::from_fn( |_| None ),
            anims: core::array::from_fn( |_| None ),
            tilemaps: core::array::from_fn( |_| None ),
            // Generates all tile rects
            rects: array::from_fn( |i| {
                let tile_x = i * S::TILE_WIDTH as usize;
                let x = (tile_x % S::ATLAS_WIDTH) as u8;
                let y = ((tile_x / S::ATLAS_WIDTH) * S::TILE_HEIGHT as usize) as u8;
                Rect{
                    x, y,
                    w:S::TILE_WIDTH,
                    h:S::TILE_HEIGHT
                }
            }),
        }
    }


    pub fn width(&self) -> usize { S::ATLAS_WIDTH }


    pub fn height(&self) -> usize { S::ATLAS_HEIGHT }


    pub fn tile_width(&self) -> u8 { S::TILE_WIDTH }


    pub fn tile_height(&self) -> u8 { S::TILE_HEIGHT }

    // TODO: Incorporate into "load_tileset"?
    pub fn load_palettes_from_atlas(&mut self, atlas:&Atlas<S, TilesetEnum, PaletteEnum>) {
        for (i, palette) in self.palettes.iter_mut().enumerate() {
            *palette = atlas.palettes[i].clone();
        };
    }

    
    pub fn pop_tileset(&mut self) {
        if let Some(ref mut top_index) = self.partition_top {
            let Some(partition) = &self.partitions[*top_index as usize] else { unreachable!() };
            let previous = partition.previous;

            self.partitions[*top_index as usize] = None;

            if let Some(previous) = previous {
                *top_index = previous;
            } else {
                self.partition_top = None;
            }   
        }
    }


    pub fn load_tileset(&mut self, atlas:&Atlas<S, TilesetEnum, PaletteEnum>, tileset_id:TilesetEnum) {
        let id:usize = tileset_id.into();
        let tileset = &atlas.tilesets[id];

        // Create new partition for tileset
        let partition = if let Some(top_index) = self.partition_top {
            let Some(top) = &self.partitions[top_index as usize] else { unreachable!() };
             Partition {
                id: tileset_id.into(),
                previous: Some(top_index),
                tiles_start_index: top.tiles_start_index + top.tiles_len as u16,
                fonts_start_index: top.fonts_start_index + top.fonts_len,
                anims_start_index: top.anims_start_index + top.anims_len,
                tilemaps_start_index: top.tilemaps_start_index +  top.tilemaps_len,
                tiles_len: tileset.tile_count,
                fonts_len: tileset.font_count,
                anims_len: tileset.anim_count,
                tilemaps_len: tileset.tilemap_count,
                debug_palette: tileset.debug_palette,
            }
        } else {
            Partition {
                id: tileset_id.into(),
                previous: None,
                tiles_start_index: 0,
                fonts_start_index: 0,
                anims_start_index: 0,
                tilemaps_start_index: 0,
                tiles_len: tileset.tile_count,
                fonts_len: tileset.font_count,
                anims_len: tileset.anim_count,
                tilemaps_len: tileset.tilemap_count,
                debug_palette: tileset.debug_palette,
            }
        };
        // println!("Partition added: {:?}", partition);

        // Copying pixels has to be tile-formatted, otherwise tile rows that end halfway through don't copy correctly
        // TODO: I use this conversion in more than one place (here and in renderer debug view), so convert it to function
        let columns = S::ATLAS_WIDTH / S::TILE_WIDTH as usize;
        for t in 0 .. partition.tiles_len as usize {
            let source_col = t % columns;
            let source_row = t / columns;
            let dest_col = (t + partition.tiles_start_index as usize) % columns;
            let dest_row = (t + partition.tiles_start_index as usize) / columns;
            let source_x = source_col * S::TILE_WIDTH as usize;
            let source_y = source_row * S::TILE_HEIGHT as usize;
            let dest_x = dest_col * S::TILE_WIDTH as usize;
            let dest_y = dest_row * S::TILE_HEIGHT as usize;
            for y in 0 .. S::TILE_HEIGHT as usize {
                for x in 0 .. S::TILE_WIDTH as usize {
                    let source_index = ((source_y + y) * S::ATLAS_WIDTH) + source_x + x;
                    let dest_index = ((dest_y + y) * S::ATLAS_WIDTH) + dest_x + x;
                    self.pixels[dest_index] = tileset.pixels[source_index];
                }
            }
        }
        
        for i in 0 .. tileset.font_count as usize {
            self.fonts[i + partition.fonts_start_index as usize] = tileset.fonts[i].clone();
        }

        for i in 0 .. tileset.anim_count as usize {
            self.anims[i + partition.anims_start_index as usize] = tileset.anims[i].clone();
        }

        for i in 0 .. tileset.tilemap_count as usize {
            self.tilemaps[i + partition.tilemaps_start_index as usize] = tileset.tilemaps[i].clone();
        }

        self.partition_top = Some(tileset_id.into());
        self.partitions[id] = Some( partition );
    }


    fn get_partition(&self, tileset_id:impl Into<usize>) -> &Partition {
        let id:usize = tileset_id.into();
        let Some(partition) = &self.partitions[id] else {
            panic!("Renderer error: Tileset {} not loaded", id)
        };
        partition
    }


    pub fn get_tileset_palette(&self, tileset_id:impl Into<usize>) -> &Palette<S> {
        let partition = self.get_partition(tileset_id);
        &self.palettes[partition.debug_palette as usize]
    }


    pub fn get_tilemap(&self, tileset_id:impl Into<usize>, tilemap_id:impl Into<usize>) -> &Tilemap {
        let partition = self.get_partition(tileset_id);
        // Calculate index from partition
        let map_id:usize = tilemap_id.into();
        let index = (partition.tilemaps_start_index as usize) + map_id;
        // Return if valid
        if let Some(tilemap) = &self.tilemaps[index]{
            tilemap
        } else {
            panic!("Renderer error: invalid tilemap ({})", map_id)
        }
    }


    pub fn get_tilemap_mut(&mut self, tileset_id:impl Into<usize>, tilemap_id:impl Into<usize>) -> &mut Tilemap {
        let partition = self.get_partition(tileset_id);
        // Calculate index from partition
        let map_id:usize = tilemap_id.into();
        let index = (partition.tilemaps_start_index as usize) + map_id;
        // Return if valid
        if let Some(tilemap) = &mut self.tilemaps[index]{
            tilemap
        } else {
            panic!("Renderer error: invalid tilemap ({})", map_id)
        }
    }


    pub fn get_font(&self, tileset_id:impl Into<usize>, font_id:impl Into<usize>) -> &Font {
        let partition = self.get_partition(tileset_id);
        // Calculate index from partition
        let font_id:usize = font_id.into();
        let index = (partition.fonts_start_index as usize) + font_id;
        // Return if valid
        if let Some(font) = &self.fonts[index]{
            font
        } else {
            panic!("Renderer error: invalid Font ({})", font_id)
        }
    }


    pub fn get_anim(&self, tileset_id:impl Into<usize>, anim_id:impl Into<usize>) -> &Anim {
        let partition = self.get_partition(tileset_id);
        // Calculate index from partition
        let anim_id:usize = anim_id.into();
        let index = (partition.anims_start_index as usize) + anim_id;
        // Return if valid
        if let Some(anim) = &self.anims[index]{
            anim
        } else {
            panic!("Renderer error: invalid Anim ({})", anim_id)
        }
    }


    pub fn get_tile(&self, index:u8, tileset_id:usize) -> TileID {
        let partition = self.get_partition(tileset_id);
        TileID(partition.tiles_start_index + index as u16)
    }


    pub fn get_rect(&self, index:usize) -> Rect<u8> {
        self.rects[index]
    }


    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * S::ATLAS_WIDTH) + x;
        self.pixels[index]
    }

}