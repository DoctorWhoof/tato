use tato_video::*;

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
    pub fn empty(palette: &mut PaletteBuilder) -> Self {
        let pixels = vec![0_u8; TILE_PIXEL_COUNT * 4];
        PalettizedImg {
            // asset_name,
            frames_h: 1,
            frames_v: 1,
            cols_per_frame: 1,
            rows_per_frame: 1,
            width: 8,
            pixels: Self::palletize(pixels.as_slice(), 8, 8, palette),
        }
    }

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
        let img_bytes = std::fs::read(file_name).unwrap();
        let img_header =
            minipng::decode_png_header(img_bytes.as_slice()).expect("Invalid PNG file");
        let capacity = img_header.required_bytes_rgba8bpc();
        let mut img_buf = vec![0u8; capacity];
        let mut img_rgba = minipng::decode_png(img_bytes.as_slice(), img_buf.as_mut_slice())
            .expect("Failed converting PNG file to pixels");

        match img_rgba.color_type() {
            //  do nothing
            minipng::ColorType::Rgba => {},
            _ => {
                // convert to rgba, 8 bits per pixel if necessary
                img_rgba.convert_to_rgba8bpc().expect("Failed converting to RGBA");
            },
        }

        let (width, height) = (img_rgba.width() as usize, img_rgba.height() as usize);
        if ((width % TILE_SIZE as usize) != 0) || ((height % TILE_SIZE as usize) != 0) {
            panic!("Build error: PNG image cannot fit into {}x{} tiles!", TILE_SIZE, TILE_SIZE)
        }

        let cols_per_frame = (img_rgba.width() / frames_h as u32) / TILE_SIZE as u32;
        let rows_per_frame = (img_rgba.height() / frames_v as u32) / TILE_SIZE as u32;

        println!("cargo:warning= Tilifying '{}'", asset_name);
        println!("cols per frame: {}, rows per frame: {}", cols_per_frame, rows_per_frame);
        println!("frames_h: {}, frames_v: {}", frames_h, frames_v);

        PalettizedImg {
            // asset_name,
            frames_h,
            frames_v,
            cols_per_frame: u8::try_from(cols_per_frame).unwrap(),
            rows_per_frame: u8::try_from(rows_per_frame).unwrap(),
            width,
            pixels: Self::palletize(
                img_rgba.pixels(),
                img_rgba.width(),
                img_rgba.height(),
                palette,
            ),
        }
    }

    // Creates a palettized image from a source RGBA image, populates the palette.
    // pub fn palletize(img: ImageData, palette: &mut PaletteBuilder) -> Vec<u8> {
    pub fn palletize(
        img_pixels: &[u8],
        width: u32,
        height: u32,
        palette: &mut PaletteBuilder,
    ) -> Vec<u8> {
        let mut pixels = vec![];
        for y in 0..height as usize {
            for x in 0..width as usize {
                let color_index = {
                    let index = x + (y * width as usize);
                    let buf_index = index * 4;
                    let r = img_pixels[buf_index];
                    let g = img_pixels[buf_index + 1];
                    let b = img_pixels[buf_index + 2];
                    let a = img_pixels[buf_index + 3];

                    let rgb_color = if a < 255 {
                        // Ensures all transp. color_map are always the same in the hashmap.
                        RGBA12::with_transparency(0, 0, 0, 0)
                    } else {
                        let color_32bit = RGBA32 { r, g, b, a };
                        RGBA12::from(color_32bit)
                    };

                    // Result
                    if palette.rgb_to_index.contains_key(&rgb_color) {
                        *palette.rgb_to_index.get(&rgb_color).unwrap()
                    } else {
                        // TODO: Error message here if palette is too large
                        let color_head = u8::try_from(palette.rgb_to_index.len()).ok().unwrap();
                        println!(
                            "cargo:warning= Inserting Palette {:02} -> {:02}: {},{},{}",
                            palette.id(),
                            color_head,
                            rgb_color.r(),
                            rgb_color.g(),
                            rgb_color.b()
                        );
                        palette.rgb_to_index.insert(rgb_color, color_head);
                        palette.push(rgb_color);
                        color_head
                    }
                };
                pixels.push(color_index)
            }
        }
        pixels
    }
}
