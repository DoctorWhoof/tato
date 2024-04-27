use core::marker::PhantomData;

use crate::*;
use alloc::{vec, vec::Vec};

// const FONT_CAPACITY:usize = 4;
// const ANIM_CAPACITY:usize = 32;
// const TILEMAP_CAPACITY:usize = 8;

/// Loads and stores fixed size tiles organized into tilesets that can be added and removed individually.
pub struct Renderer<T, P>
where T:TilesetEnum, P:PaletteEnum {
    pub(crate) palettes:            Vec<Option<Palette>>,
    rect_coords:                    Vec<Vec2<u16>>,
    specs:                          Specs,

    pub(crate) tile_indices:        BlockPool<u16>,
    tile_pixels:                    BlockPool<u8>,
    fonts:                          BlockPool<Font>,
    anims:                          BlockPool<Anim>,
    tilemaps:                       BlockPool<Tilemap>,

    tileset_marker:                 PhantomData<T>,
    palette_marker:                 PhantomData<P>,
}


impl<T, P> Renderer<T, P>
where T:TilesetEnum, P:PaletteEnum {
    pub(crate) fn new(specs:Specs) -> Self {

        let tile_count = (specs.atlas_width as usize * specs.atlas_height as usize) / (specs.tile_width as usize * specs.tile_height as usize);
        let tile_count = u16::try_from(tile_count)
            .expect("Renderer Error: Tile count can't be higher than 65535");

        #[cfg(feature = "std")]{
            println!("Renderer: Creating new Renderer with {} tiles.", tile_count);
        }

        let tileset_count = u8::try_from(T::count())
            .expect("Renderer Error: Tileset enum count can't be higher than 255");

        Renderer {
            palettes: vec![None; T::count()],
            specs,
            tileset_marker: Default::default(),
            palette_marker: Default::default(),

            // Generates all tile rects
            rect_coords: (0 .. 256).map( |i| {
                let tile_x = i * specs.tile_width as usize;
                let x = u16::try_from(tile_x % specs.atlas_width as usize).unwrap();
                let y = u16::try_from((tile_x / specs.atlas_height as usize) * specs.tile_height as usize).unwrap();
                Vec2{ x, y }
            }).collect(),

            tile_pixels: BlockPool::new(specs.atlas_width as usize * specs.atlas_height as usize, tileset_count, 0),
            tile_indices: BlockPool::new(tile_count as usize, tileset_count, 0),
            fonts: BlockPool::new(specs.max_loaded_fonts.into(), tileset_count, Font::non_init()),
            anims: BlockPool::new(specs.max_loaded_anims.into(), tileset_count, Anim::non_init()),
            tilemaps: BlockPool::new(specs.max_loaded_tilemaps.into(), tileset_count, Tilemap::non_init()),
        }
    }


    pub fn width(&self) -> u16 { self.specs.render_width }


    pub fn height(&self) -> u16 { self.specs.render_height }


    pub fn tile_width(&self) -> u8 { self.specs.tile_width }


    pub fn tile_height(&self) -> u8 { self.specs.tile_height }


    // Does not reset pixels (seems unnecessary?)
    pub fn reset(&mut self) {
        self.tile_pixels.clear();
        self.tile_indices.clear();
        self.fonts.clear();
        self.anims.clear();
        self.tilemaps.clear();
    }


    pub fn remove_tileset(&mut self, index:impl Into<u8>) -> Result<(), &'static str> {
        let index:u8 = index.into();
        self.tile_pixels.remove_block(index)?;
        self.tile_indices.remove_block(index)?;
        self.fonts.remove_block(index)?;
        self.anims.remove_block(index)?;
        self.tilemaps.remove_block(index)?;
        Ok(())
    }


    // TODO: return result
    pub fn load_tileset(&mut self, atlas:&Atlas<T,P>, tileset_id:impl TilesetEnum) -> Result<(), &'static str> {
        let id:u8 = tileset_id.into();
        let tileset = &atlas.tilesets[id as usize];
        let palette = &atlas.palettes[tileset.debug_palette as usize];

        // println!("loading tileset: {}", id);
        // println!("\npixels: {}", self.tile_pixels.data.len());
        
        if let Err(msg) = self.tile_pixels.init_block(id, tileset.pixels.len(), 0){
            panic!("Renderer: Error loading tileset pixels: '{msg}'");
        };

        if let Err(msg) = self.tile_indices.init_block(id, tileset.tile_count.into(), 0){
            panic!("Renderer: Error loading tileset pixels: '{msg}'");
        }
        
        if let Err(msg) = self.fonts.init_block(id, tileset.fonts().len(), Font::non_init()){
            panic!("Renderer: Error loading tileset fonts: '{msg}'");
        }
        
        if let Err(msg) = self.anims.init_block(id, tileset.anims().len(), Anim::non_init()){
            panic!("Renderer: Error loading tileset anims: '{msg}'");
        }
        
        if let Err(msg) = self.tilemaps.init_block(id, tileset.tilemaps().len(), Tilemap::non_init()){
            panic!("Renderer: Error loading tileset tilemaps: '{msg}'");
        }

        let block_start = if let Some(block) = self.tile_indices.get_block(tileset_id.into()){
            block.start
        } else {
            panic!("Renderer: Error, block {} not initialized", id)
        };

        // Copying pixels has to be tile-formatted, otherwise tile rows that end halfway through don't copy correctly
        // TODO: I use this conversion in more than one place (here and in renderer debug view), so convert it to a function?
        // println!("tileset pixels: {}", tileset.pixels.len());

        let dest_columns = self.specs.atlas_width as usize / self.specs.tile_width as usize;
        let tile_size = self.specs.tile_width as usize * self.specs.tile_height as usize;
        // println!("Tiles...");
        for t in 0 .. tileset.tile_count as usize {
            // println!("{t}");
            let _ = self.tile_indices.add_item_to_block(tileset_id.into(), (t + block_start) as u16);
            let dest_col = (t + block_start) % dest_columns;
            let dest_row = (t + block_start) / dest_columns;
            let dest_x = dest_col * self.specs.tile_width as usize;
            let dest_y = dest_row * self.specs.tile_height as usize;
            let mut source_px = t * tile_size;
            for y in 0 .. self.specs.tile_height as usize {
                for x in 0 .. self.specs.tile_width as usize {
                    let dest_index = ((dest_y + y) * self.specs.atlas_width as usize) + dest_x + x;
                    let source_pixel = tileset.pixels[source_px];
                    self.tile_pixels.data[dest_index] = source_pixel;
                    source_px += 1;
                }
            }
        }
        // println!();

        self.palettes[id as usize] = Some( palette.clone() );

        // TODO: returns results
        for i in 0 .. tileset.fonts().len() {
            self.fonts.add_item_to_block(tileset_id.into(), tileset.fonts()[i].clone())?
        }

        for i in 0 .. tileset.anims().len() {
            self.anims.add_item_to_block(tileset_id.into(), tileset.anims()[i].clone())?
        }

        for i in 0 .. tileset.tilemaps().len() {
            self.tilemaps.add_item_to_block(tileset_id.into(), tileset.tilemaps()[i].clone())?
        }

        Ok(())
    }


    // TODO: Get rid of all these getters, access members directly

    pub fn get_tilemap(&self, tileset_id:impl Into<u8>, tilemap_id:impl Into<usize>) -> &Tilemap {
        self.tilemaps.get(tileset_id.into(), tilemap_id.into()).unwrap()
    }


    pub fn get_tilemap_mut(&mut self, tileset_id:impl Into<u8>, tilemap_id:impl Into<usize>) -> &mut Tilemap {
        self.tilemaps.get_mut(tileset_id.into(), tilemap_id.into()).unwrap()
    }


    pub fn get_font(&self, tileset_id:impl Into<u8>, font_id:impl Into<usize>) -> &Font {
        self.fonts.get(tileset_id.into(), font_id.into()).unwrap()
    }


    pub fn get_font_mut(&mut self, tileset_id:impl Into<u8>, font_id:impl Into<usize>) -> &mut Font {
        self.fonts.get_mut(tileset_id.into(), font_id.into()).unwrap()
    }


    pub fn get_anim(&self, tileset_id:impl Into<u8>, anim_id:impl Into<usize>) -> &Anim {
        self.anims.get(tileset_id.into(), anim_id.into()).unwrap()
    }


    pub fn get_anim_mut(&mut self, tileset_id:impl Into<u8>, anim_id:impl Into<usize>) -> &mut Anim {
        self.anims.get_mut(tileset_id.into(), anim_id.into()).unwrap()
    }


    pub fn get_tile(&self, index:impl Into<usize>, tileset_id:impl Into<u8>) -> TileID {
        let tile = self.tile_indices.get(tileset_id.into(), index.into()).unwrap();
        TileID(*tile)
    }


    pub fn get_rect(&self, index:impl Into<usize>) -> Rect<u16> {
        let coord = self.rect_coords[index.into()];
        Rect {
            x: coord.x,
            y: coord.y,
            w: self.specs.tile_width as u16,
            h: self.specs.tile_height as u16
        }
    }


    pub fn get_tileset_palette(&self, tileset_id:impl Into<usize>) -> &Palette {
        let tileset_id = tileset_id.into();
        let Some(palette) = &self.palettes[tileset_id] else {
            panic!("Renderer error: Palette id not initialized: {}", tileset_id)
        };
        palette
    }


    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * self.specs.atlas_width as usize) + x;
        self.tile_pixels.data[index]
    }

}
