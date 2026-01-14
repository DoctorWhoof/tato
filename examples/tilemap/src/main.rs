use tato::{
    arena::{Arena, ArenaOps},
    prelude::*,
};
use tato_raylib::*;

mod patch;
use patch::*;

mod smileys;
use smileys::*;

const MAP_LEN: usize = 1024;

// Rects use "number of tiles" as the dimensions
fn main() -> TatoResult<()> {
    let mut frame_arena = Arena::<65_536, u32>::new();
    let mut bg_map = Tilemap::<MAP_LEN>::new(32, 32);
    let mut dash = Dashboard::new().unwrap();
    let mut tato = Tato::new(240, 180, 60);
    let mut banks = [Bank::new()];

    tato.video.bg_color = RGBA12::new(1, 2, 3);
    tato.video.wrap_bg = true;

    // Combine multiple banks into bank 0
    banks[0].add_tile(&Tile::default());
    let patch_offset = banks[0].append(&BANK_PATCH).expect("Failed to append patch bank");
    let smileys_offset =
        banks[0].append(&BANK_SMILEYS).expect("Failed to append smileys bank");

    // Display bank info to verify color deduplication
    println!("Combined bank stats:");
    println!("  Tiles: {}/{}", banks[0].tile_count(), banks[0].tile_capacity());
    println!("  Colors: {}/{}", banks[0].color_count(), COLORS_PER_PALETTE);
    println!("  Color mappings: {}/{}", banks[0].color_mapping_count(), COLOR_MAPPING_COUNT);

    // tato.load_bank(0, &bank);

    // Create offset tilemaps for patch and smileys
    let mut patch_map_offsetted = PATCH_MAP.clone();
    for cell in &mut patch_map_offsetted.cells {
        cell.id = TileID(cell.id.0 + patch_offset);
    }

    let mut smileys_map_offsetted = SMILEYS_MAP.clone();
    for cell in &mut smileys_map_offsetted.cells {
        cell.id = TileID(cell.id.0 + smileys_offset);
    }

    // Draw using the new direct tilemap API
    tato.draw_patch_3x3(&mut bg_map, Rect { x: 1, y: 1, w: 20, h: 4 }, &patch_map_offsetted);
    tato.draw_tilemap_to_tilemap(
        &mut bg_map,
        Some(Rect { x: 3, y: 5, w: 16, h: 10 }),
        &smileys_map_offsetted,
        None,
    );

    // Backend
    let mut backend = RayBackend::new(&tato);
    backend.set_bg_color(RGBA32::BLACK);

    while !backend.ray.window_should_close() {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);
        dash.frame_start(&mut frame_arena, &mut backend);
        tato.frame_start(backend.ray.get_frame_time());

        if tato.pad.is_down(Button::Right) {
            tato.video.scroll_x += 1;
        } else if tato.pad.is_down(Button::Left) {
            tato.video.scroll_x -= 1;
        }

        if tato.pad.is_down(Button::Down) {
            tato.video.scroll_y += 1;
        } else if tato.pad.is_down(Button::Up) {
            tato.video.scroll_y -= 1;
        }

        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &banks, &tato, &mut backend);
        backend.frame_present(&mut frame_arena, &tato, &banks, &[&bg_map]);
    }
    Ok(())
}
