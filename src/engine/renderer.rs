use crate::*;
use core::{usize, array};

pub const RENDER_WIDTH:usize = 288;
pub const RENDER_HEIGHT:usize = 216;
pub const RENDER_LEN:usize = RENDER_WIDTH * RENDER_HEIGHT;
pub const HUD_HEIGHT:usize = 16;

// The transparent index is hard coded to 255! Allows for black to be 0 and white is 15 in each subpalette.
pub const COLOR_TRANSPARENCY:u8 = 255;
pub const COLOR_ENTITY_RECT:u8 = 254;
pub const COLOR_COLLIDER:u8 = 253;

pub struct Renderer {
    pub pixels: [u8; RENDER_LEN],
    pub palette: [Color; 256],
    pub viewport: Rect<i32>,
    // pub atlas: Atlas<ATLAS_WIDTH,ATLAS_LEN, TILE_WIDTH, TILE_HEIGHT, TILE_COUNT>,
}


impl Default for Renderer {
    
    fn default() -> Self {
        const TRANSP:usize = COLOR_TRANSPARENCY as usize;
        const RECT:usize = COLOR_ENTITY_RECT as usize;
        const COL:usize = COLOR_COLLIDER as usize;
        Renderer {
            pixels: [0; RENDER_LEN],
            palette: array::from_fn( |i| {
                match i {
                    // Debug colors
                    TRANSP => Color{r:0,g:255,b:0,a:255},
                    RECT => Color{r:0,g:255,b:255,a:255},
                    COL => Color{r:255,g:128,b:128,a:255},
                    // Default palette is 16 tone grayscale repeated over 256 indices
                    _ =>{
                        let v = ((i%16) * 17).clamp(0, 255) as u8;
                        Color::new(v,v,v,255)  
                    } 
                }
            }),
            viewport: Rect { x:0, y:0, w:RENDER_WIDTH as i32, h:(RENDER_HEIGHT - HUD_HEIGHT) as i32 }
        }
    }
}


impl Renderer {

    #[allow(unused)]
    pub fn clear(&mut self, color_index:u8) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color_index
        }
    }


    #[inline]
    pub fn draw_pixel(&mut self, x:usize, y:usize, color_index:u8){
        draw_pixel(&mut self.pixels, RENDER_WIDTH, x, y, color_index)
    }
    

    #[inline]
    pub fn draw_line(&mut self, x0:i32, y0:i32, x1:i32, y1:i32, color_index:u8) {
        // TODO: Take viewport into account
        draw_line(&mut self.pixels, RENDER_WIDTH, x0, y0, x1, y1, color_index)
    }


    pub fn get_visible_point(&self, world_coord:Vec2<i32>) -> Option<Vec2<i32>> {
        if  world_coord.x > self.viewport.x &&
            world_coord.x < self.viewport.right() &&
            world_coord.y > self.viewport.y &&
            world_coord.y < self.viewport.bottom()
        {
            return Some(Vec2 { x: world_coord.x, y:world_coord.y})
        }
        None
    }


    pub fn draw_rect(&mut self, rect:Rect<i32>, color_index:u8){
        // TODO: Take viewport into account
        let left = rect.x;
        let top = rect.y;
        let right = rect.x + rect.w - 1;
        let bottom = rect.y + rect.h - 1;
        if left > -1 && left < RENDER_WIDTH as i32 {
            draw_line(&mut self.pixels, RENDER_WIDTH, left, top, left, bottom, color_index)
        }
        if right > -1 && right < RENDER_WIDTH as i32 {
            draw_line(&mut self.pixels, RENDER_WIDTH,  right, top, right, bottom, color_index)
        }
        if top > -1 && top < RENDER_HEIGHT as i32 {
            draw_line(&mut self.pixels, RENDER_WIDTH,  left + 1, top, right - 1, top, color_index)
        }
        if bottom > -1 && bottom < RENDER_HEIGHT as i32 {
            draw_line(&mut self.pixels, RENDER_WIDTH,  left + 1, bottom, right - 1, bottom, color_index)
        }
    }


    pub fn draw_filled_rect(&mut self, rect:Rect<i32>, color:u8){
        // TODO: Take viewport into account
        fn get_visible_rect(rect:Rect<i32>) -> Rect<i32> {
            let x = i32::clamp(rect.x, 0, RENDER_WIDTH as i32 -1);
            let right = i32::clamp(rect.x + rect.w - 1, 0, RENDER_WIDTH as i32 - 1);
            let y = i32::clamp(rect.y, 0, RENDER_HEIGHT as i32 -1);
            let bottom = i32::clamp(rect.y + rect.h - 1, 0, RENDER_HEIGHT as i32 - 1);
            Rect { x, y, w: right-x, h: bottom - y }
        }

        let rect = get_visible_rect(rect);
        for y in rect.y ..= rect.bottom() {
            for x in rect.x ..= rect.right() {
               self.draw_pixel(x as usize, y as usize, color)
            }
        }
    }

}

