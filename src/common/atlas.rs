use crate::*;
use core::{mem::variant_count, array::from_fn};


pub struct Atlas <
    S: Specs,
    TilesetEnum:Into<u8> + Into<usize> + Copy,
    PaletteEnum:Into<u8> + Into<usize> + Copy,
> where
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
{
    pub(crate) tilesets:[Tileset<S>; variant_count::<TilesetEnum>()],
    pub(crate) palettes: [Palette<S>; variant_count::<PaletteEnum>()],
}



impl <
    S: Specs,
    TilesetEnum:Into<u8> + Into<usize> + Copy,
    PaletteEnum:Into<u8> + Into<usize> + Copy,
> Atlas <S, TilesetEnum, PaletteEnum>
where
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
{
    
    pub fn load(bytes:&[u8]) -> Self {
        let mut atlas = Self {
            tilesets: from_fn(|_| Tileset::new()),
            palettes: from_fn(|i| Palette::new( u8::try_from(i).unwrap() )),
        };

        let mut cursor = Cursor::new(bytes);

        for letter in ATLAS_HEADER_TEXT.as_bytes() {
            assert!(*letter== cursor.next(), "Renderer Error: Invalid header.")
        }

        // Header data
        assert!(S::TILE_WIDTH ==  cursor.next(), "Renderer Error: Tile width does not match.");  // Tile Width
        assert!(S::TILE_HEIGHT ==  cursor.next(), "Renderer Error: Tile width does not match."); // Tile Height

        // Palette Count
        assert!(                                                                                                               
            variant_count::<PaletteEnum>() as u8 ==  cursor.next(),
            "Renderer Error: Palette count does not match"
        );

        // Tileset Count
        assert!( 
            variant_count::<TilesetEnum>() == cursor.next() as usize,
            "Renderer Error: Tileset count does not match"
        );

        // Palettes
        for palette in atlas.palettes.iter_mut(){
            #[cfg(feature = "std")]{ println!("    Initializing Palette {}", palette.id); }
            palette.id = cursor.next();
            for color in palette.colors.iter_mut() {
                color.r =  cursor.next();
                color.g =  cursor.next();
                color.b =  cursor.next();
                color.a =  cursor.next();
                #[cfg(feature = "std")]{ println!("        {:?}", color); }
            }
            
        }

        // Tilesets
        for (_i, tileset) in atlas.tilesets.iter_mut().enumerate() {
            // Header text
            for letter in TILESET_HEADER_TEXT.as_bytes() {
                assert!(*letter == cursor.next(), "Renderer Error: Invalid tileset header." )
            }
            
            // TODO: Check actual amount of actually available pixels
            let pixel_count = u16::from_ne_bytes([cursor.next(), cursor.next()]);
            assert!((pixel_count as usize) < (S::ATLAS_WIDTH * S::ATLAS_HEIGHT),  "Renderer error: Tileset pixels count will overflow!");
            tileset.tile_count = cursor.next();
            tileset.font_count = cursor.next();
            tileset.anim_count = cursor.next();
            tileset.tilemap_count = cursor.next();

            // Debug palette
            tileset.debug_palette = cursor.next();
            
            // Pixels
            #[cfg(feature = "std")]{ print!("Initializing tileset {} ... ", _i); }

            // Load pixels from linear format into tile-formatted.
            let mut _pix_count:usize = 0;
            let cols = S::ATLAS_WIDTH / S::TILE_WIDTH as usize;
            for tile_index in 0 .. tileset.tile_count as usize {
                for y in 0 .. S::TILE_HEIGHT as usize {
                    for x in 0 ..S::TILE_WIDTH as usize {
                        let col = tile_index % cols;
                        let row = tile_index / cols;
                        let tile_x = col * S::TILE_WIDTH as usize;
                        let tile_y = row * S::TILE_HEIGHT as usize;
                        let dest_px = (S::ATLAS_WIDTH  * (tile_y + y)) + (tile_x + x);
                        tileset.pixels[dest_px] = cursor.next();
                        _pix_count += 1;
                    }
                }
            }
            #[cfg(feature = "std")]{ println!("{} pixels loaded", _pix_count); }

            // Fonts
            for font_idx in 0 .. tileset.font_count as usize {
                let new_font = Font::deserialize(&mut cursor);
                #[cfg(feature = "std")]{ println!("    Initializing Font {}", new_font.id); }
                tileset.fonts[font_idx] = Some(new_font);
            }

            // Anims
            for anim_idx in 0 .. tileset.anim_count as usize {
                let new_anim = Anim::deserialize(&mut cursor);
                #[cfg(feature = "std")]{ println!("    Initializing Anim {}", new_anim.id); }
                tileset.anims[anim_idx] = Some(new_anim);
            }

            // Tilemaps
            for map_idx in 0 .. tileset.tilemap_count as usize {
                let new_map = Tilemap::deserialize(&mut cursor);
                #[cfg(feature = "std")]{ println!("    Initializing Tilemap {}", new_map.id); }
                tileset.tilemaps[map_idx] = Some(new_map);
            }

        };

        atlas
    }

}
