use core::marker::PhantomData;

use crate::*;
use alloc::{vec, vec::Vec};

/// Loads and stores pixel tiles organized into tilesets that can be added and removed individually.
/// Also contains the assets associated with each tileset (fonts, anims and tilemaps).
pub struct Renderer<T, P>
where T:TilesetEnum, P:PaletteEnum {
    pub(crate) palettes:            Vec<Option<Palette>>,
    pub(crate) rect_coords:         Vec<Vec2<u16>>,
    specs:                          Specs,

    pub(crate) tile_indices:        BlockPool<u16>,
    pub(crate) tile_pixels:         BlockPool<u8>,
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
        if u16::try_from(tile_count).is_err(){
            panic!("Renderer Error: Tile count can't be higher than 65535");
        }

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
            rect_coords: (0 .. tile_count).map( |i| {
                let tile_x = i * specs.tile_width as usize;
                let x = u16::try_from(tile_x % specs.atlas_width as usize).unwrap();
                let y = u16::try_from((tile_x / specs.atlas_height as usize) * specs.tile_height as usize).unwrap();
                Vec2{ x, y }
            }).collect(),

            tile_pixels: BlockPool::new(specs.atlas_width as usize * specs.atlas_height as usize, tileset_count, 0),
            tile_indices: BlockPool::new(tile_count, tileset_count, 0),
            fonts: BlockPool::new(specs.max_loaded_fonts.into(), tileset_count, Font::non_init()),
            anims: BlockPool::new(specs.max_loaded_anims.into(), tileset_count, Anim::non_init()),
            tilemaps: BlockPool::new(specs.max_loaded_tilemaps.into(), tileset_count, Tilemap::non_init()),
        }
    }


    pub fn width(&self) -> u16 { self.specs.atlas_width }


    pub fn height(&self) -> u16 { self.specs.atlas_height }


    pub fn tile_width(&self) -> u8 { self.specs.tile_width }


    pub fn tile_height(&self) -> u8 { self.specs.tile_height }


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

        // Pixels
        let tile_len = self.specs.tile_width as usize * self.specs.tile_height as usize;
        for tile_idx in 0 .. tileset.tile_count as usize {
            self.tile_indices.add_item_to_block(tileset_id.into(), (tile_idx + block_start) as u16)?;
            for px in 0 .. tile_len {
                let start_idx = tile_idx * tile_len;
                let source_pixel = tileset.pixels[px + start_idx];
                self.tile_pixels.add_item_to_block(tileset_id.into(), source_pixel)?;
            }
        }

        self.palettes[id as usize] = Some( palette.clone() );

        // Fonts
        for i in 0 .. tileset.fonts().len() {
            self.fonts.add_item_to_block(tileset_id.into(), tileset.fonts()[i].clone())?
        }

        // Anims
        for i in 0 .. tileset.anims().len() {
            self.anims.add_item_to_block(tileset_id.into(), tileset.anims()[i].clone())?
        }

        // Tilemaps
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


    pub fn get_tile_rect(&self, index:impl Into<usize>) -> Rect<u16> {
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


    // pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
    //     let index = (y * self.specs.atlas_width as usize) + x;
    //     self.tile_pixels.data[index]
    // }

}
