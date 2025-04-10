use macroquad::{prelude::*, texture::{load_texture, Texture2D}, window::next_frame};

#[macroquad::main("Video chip")]
async fn main() {
    println!("Hello, world!");
    // let texture: Texture2D = load_texture("examples/chess.png").await.unwrap();

    loop {
        clear_background(WHITE);
        // draw_texture_ex(
        //     &texture,
        //     0.0,
        //     0.0,
        //     WHITE,
        //     DrawTextureParams {
        //         dest_size: Some(vec2(screen_width(), screen_height())),
        //         ..Default::default()
        //     },
        // );

        next_frame().await
    }
}
