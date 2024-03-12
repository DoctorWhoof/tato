use crate::*;
use core::marker::PhantomData;
use alloc::{vec, vec::Vec};


pub struct Atlas<T, P>
where T:EnumID, P:EnumID,
{
    pub(crate) tilesets:    Vec<Tileset>,
    pub(crate) palettes:    Vec<Palette>,
    tileset_marker:         PhantomData<T>,
    palette_marker:         PhantomData<P>,
    // specs: Specs
}



impl<T, P> Atlas<T, P>
where T:EnumID, P:EnumID,
{
    
    pub fn load(specs: Specs, bytes:&[u8]) -> Self {

        let mut cursor = Cursor::new(bytes);

        for letter in ATLAS_HEADER_TEXT.as_bytes() {
            assert!(*letter== cursor.advance(), "Atlas Error: Invalid header.")
        }
        
        // Header data
        let tile_width = cursor.advance();
        let tile_height = cursor.advance();
        if tile_height != specs.tile_height || tile_width != specs.tile_width {
            panic!(
                "Atlas error: Invalid tile dimensions. Expected {}x{}, found {}x{}",
                specs.tile_width, specs.tile_height, tile_width, tile_height
            );
        }

        let mut atlas = Self {
            tilesets: vec![],
            palettes: vec![],
            palette_marker: Default::default(),
            tileset_marker: Default::default(),
            // specs
        };

        // Palette Count
        let palette_count =  cursor.advance() as usize; 

        // Tileset Count
        let tileset_count = cursor.advance() as usize;

        // Palettes
        for p in 0 .. palette_count {
            #[cfg(feature = "std")]{ println!("    Initializing Palette {}", p); }
            let palette_id = cursor.advance();
            if palette_id as usize != p {
                panic!("Atlas error: Palettes saved in atlas can't have gaps! Looked for index {}, found {}", p, palette_id)
            }
            // let mut palette = vec![Color::default(); atlas.specs.colors_per_palette as usize];
            let mut palette = Palette::new(specs, palette_id);
            for color in palette.colors.iter_mut() {
                color.r =  cursor.advance();
                color.g =  cursor.advance();
                color.b =  cursor.advance();
                color.a =  cursor.advance();
                #[cfg(feature = "std")]{ println!("        {:?}", color); }
            }
            atlas.palettes.push(palette);
        }

        // Tilesets
        for _i in 0 .. tileset_count {
            // Header text
            for letter in TILESET_HEADER_TEXT.as_bytes() {
                assert!(*letter == cursor.advance(), "Atlas Error: Invalid tileset header." )
            }

            // TODO: Check actual amount of actually available pixels
            let atlas_width = u16::from_ne_bytes([cursor.advance(), cursor.advance()]);
            let pixel_count = u16::from_ne_bytes([cursor.advance(), cursor.advance()]) as usize;
            assert!(pixel_count < (specs.atlas_width as usize * specs.atlas_height as usize),  "Atlas error: Tileset pixels count will overflow!");

            let tile_count = cursor.advance();
            let font_count = cursor.advance() as usize;
            let anim_count = cursor.advance() as usize;
            let tilemap_count = cursor.advance() as usize;

            // New tileset
            #[cfg(feature = "std")]{ println!("Atlas: Initializing tileset {} with {} pixels, atlas width = {} ... ", _i, pixel_count, atlas_width); }
            let mut tileset = Tileset::new(pixel_count, tile_count);

            // Debug palette
            tileset.debug_palette = cursor.advance();
            
            // Pixels (linear formatted)
            for px_index in 0 .. pixel_count {
                tileset.pixels[px_index] = cursor.advance()
            }

            // Verify that we reached the next section header (all pixels loaded)
            for letter in "fonts".as_bytes() {
                assert!(*letter== cursor.advance(), "Atlas Error: Invalid fonts header.")
            }
            #[cfg(feature = "std")]{ println!("    {} pixels loaded!", pixel_count); }

            // Fonts
            for _ in 0 .. font_count {
                let new_font = Font::deserialize(&mut cursor);
                #[cfg(feature = "std")]{ println!("    Initializing Font {}", new_font.id); }
                tileset.push_font(new_font);
            }

            // Anims
            for letter in "anims".as_bytes() {
                assert!(*letter== cursor.advance(), "Atlas Error: Invalid anims header.")
            }
            for _ in 0 .. anim_count {
                let new_anim = Anim::deserialize(&mut cursor);
                #[cfg(feature = "std")]{ println!("    Initializing Anim {}", new_anim.id); }
                tileset.push_anim(new_anim);
            }

            // Tilemaps
            for letter in "tilemaps".as_bytes() {
                assert!(*letter== cursor.advance(), "Atlas Error: Invalid tilemaps header.")
            }
            for _ in 0 .. tilemap_count {
                let new_map = Tilemap::deserialize(&mut cursor);
                #[cfg(feature = "std")]{ println!("    Initializing Tilemap {}", new_map.id); }
                tileset.push_tilemap(new_map);
            }

            atlas.tilesets.push(tileset);
        };

        atlas
    }

}
