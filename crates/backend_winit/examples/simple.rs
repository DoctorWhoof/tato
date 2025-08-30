use tato::prelude::*;
use tato_winit::WinitBackend;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    // Create a simple Tato instance
    let mut tato = Tato::new(240, 180, 60);
    let mut frame_arena = tato::arena::Arena::<307_200, u32>::new();

    // Create a simple background tilemap
    let mut bg = Tilemap::<1600>::new(30, 22);

    // Set up basic video settings
    tato.video.bg_color = RGBA12::new(2, 4, 7);

    // Load default tileset which includes basic tiles
    let _tileset = tato.push_tileset(0, DEFAULT_TILESET).expect("Failed to load tileset");

    // Set up background with default tiles
    for row in 0..bg.rows() as i16 {
        for col in 0..bg.columns() as i16 {
            // Use checkers and arrows from default tiles
            let tile_id = if (col + row) % 2 == 0 {
                TILE_CHECKERS
            } else {
                TILE_ARROW
            };

            bg.set_cell(BgOp {
                col,
                row,
                tile_id,
                flags: TileFlags::default(),
                sub_palette: PaletteID(0),
            });
        }
    }

    // Create the winit backend
    let backend = WinitBackend::new(&tato, &mut frame_arena).await;

    // Player position
    let mut player_x = 120.0f32;
    let mut player_y = 90.0f32;

    // Run the main loop
    backend.run(move |backend| {
        // Start frame
        frame_arena.clear();
        backend.frame_start(&mut frame_arena);
        tato.frame_start(1.0 / 60.0);

        // Update input
        backend.update_input(&mut tato.pad);

        // Simple player movement
        let speed = 2.0;
        if tato.pad.is_down(Button::Left) {
            player_x -= speed;
        }
        if tato.pad.is_down(Button::Right) {
            player_x += speed;
        }
        if tato.pad.is_down(Button::Up) {
            player_y -= speed;
        }
        if tato.pad.is_down(Button::Down) {
            player_y += speed;
        }

        // Draw a smiley sprite
        tato.video.draw_fg_tile(DrawBundle {
            x: player_x as i16,
            y: player_y as i16,
            id: TILE_SMILEY,
            flags: TileFlags::default(),
            sub_palette: PaletteID(0),
        });

        // Clear with background color
        backend.clear(RGBA32 { r: 32, g: 64, b: 128, a: 255 });

        // Finish frame processing
        tato.frame_finish();

        // Present frame with the background tilemap
        backend.frame_present(&mut frame_arena, &tato, &[&bg]);

        // Check if we should exit
        if backend.get_pressed_key() == Some(Key::Enter) {
            println!("Enter pressed, exiting...");
        }
    });
}
