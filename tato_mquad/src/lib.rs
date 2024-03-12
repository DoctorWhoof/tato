
use std::{marker::PhantomData, time::Instant, vec};
use macroquad::prelude::*;
use tato::{PaletteEnum, TilesetEnum, World};


pub struct App<T, P>
where T:TilesetEnum, P:PaletteEnum
{
    pub overlay_position: Vec2,
    pub overlay_line_spacing: f32,
    overlay: Vec<String>,
    render_image: Image,
    render_texture: Texture2D,
    time: Instant,
    tileset_marker: PhantomData<T>,
    palette_marker: PhantomData<P>
}


impl<T,P> App<T,P>
where T:TilesetEnum, P:PaletteEnum {

    pub fn new(world:&World<T,P>) -> Self
    where T:TilesetEnum, P:PaletteEnum {
        let render_image =  Image::gen_image_color( world.framebuf.width(), world.framebuf.height(), BLACK);
        let render_texture = Texture2D::from_image(&render_image);
        render_texture.set_filter(FilterMode::Nearest);

        Self {
            overlay_position: Vec2::new(10.0, 20.0),
            overlay_line_spacing: 16.0,
            overlay: vec![],
            render_image,
            render_texture,
            time: Instant::now(),
            tileset_marker: Default::default(),
            palette_marker: Default::default()
        }
    }


    pub fn start_frame(&mut self, world:&mut World<T,P>)
    where T:TilesetEnum, P:PaletteEnum {
        world.start_frame(self.time.elapsed().as_secs_f32());
    }


    pub fn finish_frame(&mut self, world:&mut World<T,P>)
    where T:TilesetEnum, P:PaletteEnum {
        // Render scaling pre-calc
        let width = world.framebuf.width();
        let height = world.framebuf.height();
        let scale = (screen_height() / height as f32).floor();
        let render_width = width as f32 * scale;
        let render_height = height as f32 * scale;
        let draw_rect_x = (screen_width() - render_width) / 2.0;
        let draw_rect_y = (screen_height() - render_height) / 2.0;

        // Copy from framebuffer to macroquad texture
        let source = world.framebuf.pixels();
        for y in 0 .. height {
            for x in 0 .. width {
                let source_index = (y as usize * width as usize) + x as usize;
                let color = source[source_index];
                self.render_image.set_pixel(
                    x as u32,
                    y as u32,
                    Color::from_rgba(color.r, color.g, color.b, color.a),
                )
            }
        }

        // Render texture to screen
        clear_background(BLACK);
        self.render_texture.update(&self.render_image);
        draw_texture_ex(
            &self.render_texture,
            draw_rect_x,
            draw_rect_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(render_width, render_height)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        // Overlay
        for (i,item) in self.overlay.iter().enumerate() {
            let y = (i as f32 * self.overlay_line_spacing) + self.overlay_position.y ;
            draw_text( item, self.overlay_position.x, y, self.overlay_line_spacing, WHITE);
        }

        // Finish (calculate timings)
        self.overlay.clear();
        world.finish_frame(self.time.elapsed().as_secs_f32());
    }


    pub fn push_overlay(&mut self, text:String) {
        self.overlay.push(text);
    }



}