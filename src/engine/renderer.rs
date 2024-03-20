use core::marker::PhantomData;

use crate::*;
use alloc::{vec, vec::Vec};

/// Loads and stores fixed size tiles organized into tilesets that can be added and removed individually.
pub struct Renderer<T, P>
where T:TilesetEnum, P:PaletteEnum,
{
    pub(crate) partitions:  Vec<Option<Partition>>,
    pub(crate) palettes:    Vec<Option<Palette>>,
    partition_top:          Option<u8>,
    fonts:                  Vec<Font>,
    anims:                  Vec<Anim>,
    tilemaps:               Vec<Tilemap>,
    pixels:                 Vec<u8>,
    rect_coords:                 Vec<Vec2<u16>>,
    tileset_marker:         PhantomData<T>,
    palette_marker:         PhantomData<P>,
    specs: Specs,
}


impl<T, P> Renderer<T, P>
where T:TilesetEnum, P:PaletteEnum,
{
    pub(crate) fn new(specs:Specs) -> Self {

        #[cfg(feature = "std")]{
            let tile_count = (specs.atlas_width as usize * specs.atlas_height as usize) / (specs.tile_width as usize * specs.tile_height as usize);
            println!("Renderer: Creating new Renderer with {} tiles.", tile_count);
        }

        Renderer {
            pixels: vec![0; specs.atlas_width as usize * specs.atlas_height as usize],
            palettes: vec![None; P::count()],
            partitions: vec![None; T::count()],
            partition_top: None,
            fonts: vec![],
            anims: vec![],
            tilemaps: vec![],
            // Generates all tile rects
            rect_coords: (0 .. 256).map( |i| {
                let tile_x = i * specs.tile_width as usize;
                let x = u16::try_from(tile_x % specs.atlas_width as usize).unwrap();
                let y = u16::try_from((tile_x / specs.atlas_height as usize) * specs.tile_height as usize).unwrap();
                Vec2{ x, y }
            }).collect(),
            specs,
            tileset_marker: Default::default(),
            palette_marker: Default::default()
        }
    }


    pub fn width(&self) -> u16 { self.specs.render_width }


    pub fn height(&self) -> u16 { self.specs.render_height }


    pub fn tile_width(&self) -> u8 { self.specs.tile_width }


    pub fn tile_height(&self) -> u8 { self.specs.tile_height }

    // TODO: Incorporate into "load_tileset"?
    pub fn load_palettes_from_atlas(&mut self, atlas:&Atlas<T,P>) {
        for (i, palette) in self.palettes.iter_mut().enumerate() {
            *palette = Some(atlas.palettes[i].clone());
        };
    }


    pub fn pop_tileset(&mut self) {
        // self.partitions.pop();
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


    pub fn load_tileset(&mut self, atlas:&Atlas<T,P>, tileset_id:impl TilesetEnum) {
        let id:u8 = tileset_id.into();
        let tileset = &atlas.tilesets[id as usize];
        let palette = &atlas.palettes[tileset.debug_palette as usize];

        // Create new partition for tileset
        let partition = if let Some(top_index) = self.partition_top {
            let Some(top) = &self.partitions[top_index as usize] else { unreachable!() };
             Partition {
                previous: Some(top_index),
                tiles_start_index: top.tiles_start_index + top.tiles_len as u16,
                fonts_start_index: top.fonts_start_index + top.fonts_len,
                anims_start_index: top.anims_start_index + top.anims_len,
                tilemaps_start_index: top.tilemaps_start_index +  top.tilemaps_len,
                tiles_len: tileset.tile_count(),
                fonts_len: tileset.font_count(),
                anims_len: tileset.anim_count(),
                tilemaps_len: tileset.tilemap_count(),
                debug_palette: tileset.debug_palette,
            }
        } else {
            Partition {
                previous: None,
                tiles_start_index: 0,
                fonts_start_index: 0,
                anims_start_index: 0,
                tilemaps_start_index: 0,
                tiles_len: tileset.tile_count(),
                fonts_len: tileset.font_count(),
                anims_len: tileset.anim_count(),
                tilemaps_len: tileset.tilemap_count(),
                debug_palette: tileset.debug_palette,
            }
        };

        // Copying pixels has to be tile-formatted, otherwise tile rows that end halfway through don't copy correctly
        // TODO: I use this conversion in more than one place (here and in renderer debug view), so convert it to a function?
        let dest_columns = self.specs.atlas_width as usize / self.specs.tile_width as usize;
        for t in 0 .. partition.tiles_len as usize {
            let dest_col = (t + partition.tiles_start_index as usize) % dest_columns;
            let dest_row = (t + partition.tiles_start_index as usize) / dest_columns;
            let dest_x = dest_col * self.specs.tile_width as usize;
            let dest_y = dest_row * self.specs.tile_height as usize;
            let mut source_px = t * (self.specs.tile_width as usize * self.specs.tile_height as usize);
            for y in 0 .. self.specs.tile_height as usize {
                for x in 0 .. self.specs.tile_width as usize {
                    let dest_index = ((dest_y + y) * self.specs.atlas_width as usize) + dest_x + x;
                    let source_pixel =  tileset.pixels[source_px];
                    self.pixels[dest_index] = source_pixel;
                    source_px += 1;
                }
            }
        }

        for i in 0 .. tileset.fonts().len() {
            self.fonts.push(tileset.fonts()[i].clone())
        }

        for i in 0 .. tileset.anims().len() {
            self.anims.push(tileset.anims()[i].clone())
        }

        for i in 0 .. tileset.tilemaps().len() {
            self.tilemaps.push(tileset.tilemaps()[i].clone())
        }

        // println!("Partition added: {:?}", partition);
        self.partition_top = Some(id);
        self.partitions[id as usize] = Some( partition );
        self.palettes[palette.id() as usize] = Some( palette.clone() );
    }


    fn get_partition(&self, tileset_id:impl Into<usize>) -> &Partition {
        let id:usize = tileset_id.into();
        let Some(partition) = &self.partitions[id] else {
            panic!("Renderer error: Tileset id not initialized: {}", id)
        };
        partition
    }


    pub fn get_tileset_palette(&self, tileset_id:impl Into<usize>) -> &Palette {
        let partition = self.get_partition(tileset_id);
        let palette_id = partition.debug_palette as usize;
        let Some(palette) = &self.palettes[palette_id] else {
            panic!("Renderer error: Palette id not initialized: {}", palette_id)
        };
        palette
    }


    pub fn get_tilemap(&self, tileset_id:impl Into<usize>, tilemap_id:impl Into<usize>) -> &Tilemap {
        let partition = self.get_partition(tileset_id);
        // Calculate index from partition
        let map_id:usize = tilemap_id.into();
        let index = (partition.tilemaps_start_index as usize) + map_id;
        // Return if valid
        if let Some(tilemap) = &self.tilemaps.get(index){
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
        if let Some(tilemap) = self.tilemaps.get_mut(index){
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
        if let Some(font) = &self.fonts.get(index){
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
        if let Some(anim) = &self.anims.get(index){
            anim
        } else {
            panic!("Renderer error: invalid Anim ({})", anim_id)
        }
    }


    pub fn get_tile(&self, index:u8, tileset_id:usize) -> TileID {
        let partition = self.get_partition(tileset_id);
        TileID(partition.tiles_start_index + index as u16)
    }


    pub fn get_rect(&self, index:usize) -> Rect<u16> {
        let coord = self.rect_coords[index];
        Rect {
            x: coord.x,
            y: coord.y,
            w: self.specs.tile_width as u16,
            h: self.specs.tile_height as u16
        }
    }


    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * self.specs.atlas_width as usize) + x;
        self.pixels[index]
    }

}
