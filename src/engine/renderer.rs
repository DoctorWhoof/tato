use crate::*;
use core::{usize, array};

pub const COLOR_TRANSPARENCY:u8 = 255; // The transparent index is hard coded to 255! Allows for black to be 0 and white is 15 in each subpalette.
pub const COLOR_ENTITY_RECT:u8 = 254;
pub const COLOR_COLLIDER:u8 = 253;


/// Allows writing pixels to a color-indexed pixel buffer.
pub struct Renderer<const PIXEL_COUNT:usize> {
    pub(super) pixels: [u8; PIXEL_COUNT],
    pub(super) palette: [Color; 256],
    pub(super) viewport: Rect<i32>,
    width: u16,
    height: u16
}

impl<const PIXEL_COUNT:usize> Renderer<PIXEL_COUNT> {

    pub fn new(width:u16, height:u16) -> Self {
        assert!(PIXEL_COUNT==width as usize * height as usize, "Renderer: Error, width x height must equal PIXEL_COUNT");
        const TRANSP:usize = COLOR_TRANSPARENCY as usize;
        const RECT:usize = COLOR_ENTITY_RECT as usize;
        const COL:usize = COLOR_COLLIDER as usize;
        Renderer {
            pixels: [0; PIXEL_COUNT],
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
            viewport: Rect { x:0, y:0, w:width as i32, h:height as i32 },
            width,
            height
        }
    }


    pub fn width(&self) -> usize { self.width as usize }


    pub fn height(&self) -> usize { self.height as usize }


    pub fn pixels(&self) -> &[u8; PIXEL_COUNT] { &self.pixels }


    pub fn palette(&self) -> &[Color; 256] { &self.palette }


    pub fn viewport(&self) -> &Rect<i32> { &self.viewport }


    #[allow(unused)]
    pub fn clear(&mut self, color_index:u8) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color_index
        }
    }


    #[inline]
    pub fn draw_pixel(&mut self, x:usize, y:usize, color_index:u8){
        draw_pixel(&mut self.pixels, self.width as usize, x, y, color_index)
    }
    

    #[inline]
    pub fn draw_line(&mut self, x0:i32, y0:i32, x1:i32, y1:i32, color_index:u8) {
        // TODO: Take viewport into account
        draw_line(&mut self.pixels, self.width as usize, x0, y0, x1, y1, color_index)
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
        if left > -1 && left < self.width as i32 {
            draw_line(&mut self.pixels, self.width as usize, left, top, left, bottom, color_index)
        }
        if right > -1 && right < self.width as i32 {
            draw_line(&mut self.pixels, self.width as usize,  right, top, right, bottom, color_index)
        }
        if top > -1 && top < self.height as i32 {
            draw_line(&mut self.pixels, self.width as usize,  left + 1, top, right - 1, top, color_index)
        }
        if bottom > -1 && bottom < self.height as i32 {
            draw_line(&mut self.pixels, self.width as usize,  left + 1, bottom, right - 1, bottom, color_index)
        }
    }


    pub fn draw_filled_rect(&mut self, rect:Rect<i32>, color:u8){
        // TODO: Take viewport into account
        let rect = {
            let x = i32::clamp(rect.x, 0, self.width as i32 -1);
            let right = i32::clamp(rect.x + rect.w - 1, 0, self.width as i32 - 1);
            let y = i32::clamp(rect.y, 0, self.height as i32 -1);
            let bottom = i32::clamp(rect.y + rect.h - 1, 0, self.height as i32 - 1);
            Rect { x, y, w: right-x, h: bottom - y }
        };
        for y in rect.y ..= rect.bottom() {
            for x in rect.x ..= rect.right() {
               self.draw_pixel(x as usize, y as usize, color)
            }
        }
    }

}

