
use macroquad::prelude::*;
use tato::{Renderer, World};

struct App {
    render_image: Image,
    render_texture: Texture2D,

}

impl App {

    pub fn new(width:u16, height:u16) -> Self {
        let render_image =  Image::gen_image_color( width, height, BLACK);
        let mut render_texture = Texture2D::from_image(&render_image);
        render_texture.set_filter(FilterMode::Nearest);
        
        Self { render_image, render_texture }
    }

    // pub fn render_world(&mut self, world:&Renderer) {

    // }

}