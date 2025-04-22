use image::{DynamicImage, ImageReader};
use tato::video::{color::Color9Bit, ColorRGB24, TILE_SIZE};

use super::*;


/// Stores a palettized version of an image as well as layout data (frame count, columns and rows per frame).
/// Uses an external palette and color map to ensure consistency accross many image sources.
#[derive(Debug)]
pub(crate) struct PalettizedImg {
    // pub asset_name: String,
    // pub frames_h: u8,
    // pub frames_v: u8,
    // pub cols_per_frame: u8,
    // pub rows_per_frame: u8,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl PalettizedImg {
    pub fn from_image(
        file_name: &str,
        // frame_layout: Option<(u8, u8)>,
        // cols_per_frame: u8,
        // rows_per_frame: u8,
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
                "cargo:warning=Image for '{}' is Rgba8, proceeding... ",
                asset_name
            );
        } else {
            println!(
                "cargo:warning=Image for '{}' is not Rgba8, converting... ",
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

        // let (cols_per_frame, rows_per_frame) = match frame_layout {
        //     Some(value) => (value.0, value.1),
        //     None => (1, 1),
        // };

        // let frames_h = u8::try_from((width / cols_per_frame as usize) / TILE_SIZE as usize)
        //     .ok()
        //     .unwrap();

        // let frames_v = u8::try_from((height / rows_per_frame as usize) / TILE_SIZE as usize)
        //     .ok()
        //     .unwrap();

        // println!(
        //     "cargo:warning=    Tilifying '{}' to {} frames with {}x{} tiles",
        //     asset_name,
        //     frames_h as usize * frames_v as usize,
        //     cols_per_frame,
        //     rows_per_frame
        // );

        println!(
            "cargo:warning=    Tilifying '{}'",
            asset_name,
        );

        PalettizedImg {
            // asset_name,
            // frames_h,
            // frames_v,
            // cols_per_frame,
            // rows_per_frame,
            width,
            height,
            pixels: Self::palletize(img_rgba, palette),
        }
    }

    // Populates the palettized, 1 byte-per-pixel image from a source RGBA image.
    pub fn palletize(
        img: DynamicImage,
        palette: &mut PaletteBuilder,
    ) -> Vec<u8> {
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
                        Color9Bit::new(0, 0, 0) // Ensures all transp. color_map are always the same in the hashmap.
                    } else {
                        let color_rgb = ColorRGB24{r,g,b};
                        Color9Bit::from(color_rgb)
                    };

                    // Result
                    if palette.color_hash.contains_key(&rgb_color) {
                        *palette.color_hash.get(&rgb_color).unwrap()
                    } else {
                        let color_head = u8::try_from(palette.color_hash.len()).ok().unwrap();
                        println!(
                            "cargo:warning=    Inserting Palette {:02} -> {:02}: {:?}",
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

// Json must be in Aseprite export format
// fn read_anim_from_json(path:PathBuf) -> Result<HashMap<String, Vec<u8>>, Error> {
//     let mut result = HashMap::new();

//     let text = std::fs::read_to_string(path)?;
//     let parsed:JsonValue = text.parse().unwrap();
//     let json: HashMap<_, _> = parsed.try_into().unwrap();

//     // Aseprite parsing
//     let meta = &json["meta"];
//     let frame_tags = &meta["frameTags"];
//     if let JsonValue::Array(tags) = frame_tags {
//         for tag in tags.iter() {
//             let JsonValue::String(name) = &tag["name"] else { break };
//             let JsonValue::Number(head) = &tag["from"] else { break };
//             let JsonValue::Number(tail) = &tag["to"] else { break };
//             let Ok(head) = u8::try_from(*head as usize) else { break };
//             let Ok(tail) = u8::try_from(*tail as usize) else { break };

//             let range = (head ..= tail).collect(); // indices start at 0! (not in atlas space)
//             result.insert(name.clone(), range);
//         }
//     }

//     Ok(result)
// }

// OLD

// pub fn convert_sprite(file_name:&str, frame_cols:usize, frame_rows:usize, sub_palette:u8) {
//     // Load image
//     let img = AtlasBuilder::from_image(file_name, Some((frame_cols, frame_rows)), sub_palette);

//     // split into tiles, remove redundant tiles and save resulting unique tiles
//     let (img_bytes, tile_ids) = tilify(&img);
//     write( format!("{ASSET_DEST}{file_name}.pix"), img_bytes.as_slice()).unwrap();

//     // Use JSON data to save each animation into a separate file
//     if let Some(anims) = img.anims {
//         for (name, anim) in anims {
//             if name.is_empty() { continue }
//             let mut tiles_per_anim = vec![];
//             let frame_size = frame_cols * frame_rows;
//             for frame in anim {
//                 let offset = frame_size * frame as usize;
//                 // Push all tiles per frame
//                 for index in 0 .. frame_size {
//                     tiles_per_anim.push(tile_ids[index + offset])
//                 }
//             }
//             write(
//                 format!("{ASSET_DEST}{file_name}_{name}.anim"),
//                 tiles_per_anim.as_slice()
//             ).unwrap();
//         }
//     }
// }

// pub fn convert_tiles(file_name:&str, sub_palette:u8, save_map:bool) {
//     // Load image
//     let img = AtlasBuilder::from_image(file_name, None, sub_palette);

//     // split into tiles, remove redundant tiles and save resulting unique tiles
//     let (img_bytes, tile_ids) = tilify(&img);
//     write( format!("{ASSET_DEST}{file_name}.pix"), img_bytes.as_slice()).unwrap();

//     if save_map {
//         write( format!("{ASSET_DEST}{file_name}.map"), tile_ids.as_slice()).unwrap();
//     }
// }
