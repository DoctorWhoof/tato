use image::{DynamicImage, ImageReader};
use tato_video::*;

use crate::*;

use super::*;

/// Stores a palettized version of an image as well as layout data (frame count, columns and rows per frame).
/// Uses an external palette and color map to ensure consistency accross many image sources.
#[derive(Debug)]
pub(crate) struct PalettizedImg {
    // pub asset_name: String,
    pub frames_h: u8,
    pub frames_v: u8,
    pub cols_per_frame: u8,
    pub rows_per_frame: u8,
    pub width: usize,
    pub pixels: Vec<u8>,
}

impl PalettizedImg {
    pub fn from_image(
        file_name: &str,
        frames_h: u8,
        frames_v: u8,
        palette: &mut PaletteBuilder,
    ) -> PalettizedImg {
        let split = file_name.split('/');
        let last = split.last().unwrap().to_string();
        let mut last_split = last.split('.');
        let asset_name = last_split.next().unwrap().to_string();

        println!("cargo:warning=Converting image {}", asset_name);
        let mut img_rgba = ImageReader::open(file_name).unwrap().decode().unwrap();
        if let DynamicImage::ImageRgba8 { .. } = img_rgba {
            println!(
                "cargo:warning= Image for '{}' is Rgba8, proceeding... ",
                asset_name
            );
        } else {
            println!(
                "cargo:warning= Image for '{}' is not Rgba8, converting... ",
                asset_name
            );
            img_rgba = DynamicImage::from(img_rgba.to_rgba8());
        }

        let (width, height) = (img_rgba.width() as usize, img_rgba.height() as usize);
        if ((width % TILE_SIZE as usize) != 0) || ((height % TILE_SIZE as usize) != 0) {
            panic!(
                "Build error: PNG image cannot fit into {}x{} tiles!",
                TILE_SIZE, TILE_SIZE
            )
        }

        let cols_per_frame = (img_rgba.width() / frames_h as u32) / TILE_SIZE as u32;
        let rows_per_frame = (img_rgba.height() / frames_v as u32) / TILE_SIZE as u32;

        println!("cargo:warning= Tilifying '{}'", asset_name);
        println!(
            "cols per frame: {}, rows per frame: {}",
            cols_per_frame, rows_per_frame
        );
        println!("frames_h: {}, frames_v: {}", frames_h, frames_v);

        PalettizedImg {
            // asset_name,
            frames_h,
            frames_v,
            cols_per_frame: u8::try_from(cols_per_frame).unwrap(),
            rows_per_frame: u8::try_from(rows_per_frame).unwrap(),
            width,
            pixels: Self::palletize(img_rgba, palette),
        }
    }

    // Populates the palettized, 1 byte-per-pixel image from a source RGBA image.
    pub fn palletize(img: DynamicImage, palette: &mut PaletteBuilder) -> Vec<u8> {
        let mut pixels = vec![];
        for y in 0..img.height() as usize {
            for x in 0..img.width() as usize {
                let color = {
                    let buf = img.as_bytes();
                    let index = x + (y * img.width() as usize);
                    let buf_index = index * 4;
                    let r = buf[buf_index];
                    let g = buf[buf_index + 1];
                    let b = buf[buf_index + 2];
                    let a = buf[buf_index + 3];

                    let rgb_color = if a < 255 {
                        Color14Bit::new(0, 0, 0, 0) // Ensures all transp. color_map are always the same in the hashmap.
                    } else {
                        let color_rgb = ColorRGB32 { r, g, b, a };
                        Color14Bit::from(color_rgb)
                    };

                    // Result
                    if palette.color_hash.contains_key(&rgb_color) {
                        *palette.color_hash.get(&rgb_color).unwrap()
                    } else {
                        let color_head = u8::try_from(palette.color_hash.len()).ok().unwrap();
                        println!(
                            "cargo:warning= Inserting Palette {:02} -> {:02}: {:?}",
                            palette.id(),
                            color_head,
                            rgb_color
                        );
                        palette.color_hash.insert(rgb_color, color_head);
                        palette.push(rgb_color);
                        color_head
                    }
                };
                pixels.push(color)
            }
        }
        pixels
    }
}
