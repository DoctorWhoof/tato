use alloc::{vec, vec::Vec};
use crate::*;

/// Allows writing pixels to a frame buffer.
pub struct FrameBuf  {
    pub(super) pixels: Vec<Pixel>,
    pub(super) viewport: Rect<i32>,
    pub(super) specs: Specs,
}

impl FrameBuf {

    pub(super) fn new(specs:Specs) -> Self {
        FrameBuf {
            pixels: vec![Pixel::default(); specs.render_width as usize * specs.render_height as usize],
            viewport: Rect { x:0, y:0, w:specs.render_width as i32, h:specs.render_height as i32 },
            specs
        }
    }


    pub fn width(&self) -> u16 { self.specs.render_width }


    pub fn height(&self) -> u16 { self.specs.render_height }


    pub fn pixels(&self) -> &[Pixel] { &self.pixels }


    pub fn viewport(&self) -> &Rect<i32> { &self.viewport }


    pub fn clear(&mut self, color:Color24) {
        for pixel in self.pixels.iter_mut() {
            pixel.color = color;
            pixel.depth = 0;
        }
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


    pub fn draw_pixel(&mut self, x:usize, y:usize, color:Color24, depth:u8){
        draw_pixel(&mut self.pixels, self.specs.render_width, x, y, color, depth)
    }


    pub fn draw_line(&mut self, x0:i32, y0:i32, x1:i32, y1:i32, color:Color24, depth:u8) {
        // TODO: Take viewport into account
        if x0 < 0 || x1 > self.specs.render_width as i32 { return }
        if y0 < 0 || y1 > self.specs.render_height as i32 { return }
        draw_line(&mut self.pixels, self.specs.render_width, x0, y0, x1, y1, color, depth)
    }


    pub fn draw_rect(&mut self, rect:Rect<i32>, color:Color24){
        // TODO: Take viewport into account
        let left = rect.x;
        let top = rect.y;
        let right = rect.x + rect.w - 1;
        let bottom = rect.y + rect.h - 1;
        if left > -1 && left < self.specs.render_width as i32 - 1 {
            draw_line(&mut self.pixels, self.specs.render_width, left, top, left, bottom - 1, color, 255)
        }
        if right > -1 && right < self.specs.render_width as i32 - 1 {
            draw_line(&mut self.pixels, self.specs.render_width,  right, top, right, bottom - 1, color, 255)
        }
        if top > -1 && top < self.specs.render_height as i32 - 1 {
            draw_line(&mut self.pixels, self.specs.render_width,  left, top, right, top, color, 255)
        }
        if bottom > -1 && bottom < self.specs.render_height as i32 - 1 {
            draw_line(&mut self.pixels, self.specs.render_width,  left, bottom, right, bottom, color, 255)
        }
    }


    pub fn draw_filled_rect(&mut self, rect:Rect<i32>, color:Color24){
        // TODO: Take viewport into account
        let rect = {
            let x = i32::clamp(rect.x, 0, self.specs.render_width as i32 -1);
            let right = i32::clamp(rect.x + rect.w - 1, 0, self.specs.render_width as i32 - 1);
            let y = i32::clamp(rect.y, 0, self.specs.render_height as i32 -1);
            let bottom = i32::clamp(rect.y + rect.h - 1, 0, self.specs.render_height as i32 - 1);
            Rect { x, y, w: right-x, h: bottom - y }
        };
        for y in rect.y ..= rect.bottom() {
            for x in rect.x ..= rect.right() {
               self.draw_pixel(x as usize, y as usize, color, 255)
            }
        }
    }

}


pub(crate) fn draw_pixel(pixels: &mut [Pixel], buffer_width:u16, x:usize, y:usize, color:Color24, depth:u8){
    let index = (y * buffer_width as usize) + x;
    if index > pixels.len() { return }
    let pixel = &mut pixels[index];
    if pixel.depth > depth { return }
    pixel.color = color;
    pixel.depth = depth;
}


#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_line(pixels: &mut [Pixel], buffer_width:u16, x0:i32, y0:i32, x1:i32, y1:i32, color:Color24, depth:u8) {

    let buffer_height = pixels.len() / buffer_width as usize;

    let mut x_head = (x0 as f32).clamp(0.0, buffer_width as f32 - 1.0);
    let mut y_head = (y0 as f32).clamp(0.0, buffer_height as f32 - 1.0);
    let x_tail = x1.clamp(0, buffer_width as i32 - 1);
    let y_tail = y1.clamp(0, buffer_height as i32 - 1);

    let w = x_tail - x_head as i32;
    let h = y_tail - y_head as i32;
    let longest = if w.abs() > h.abs() { w.abs() } else { h.abs() };
    let inc_x = w as f32 / longest as f32;
    let inc_y = h as f32 / longest as f32;

    for _ in 0 ..= longest as usize {
        draw_pixel(pixels, buffer_width, x_head as usize, y_head as usize, color, depth);
        x_head += inc_x;
        y_head += inc_y;
        if x_head as usize >= buffer_width as usize || y_head as usize >= buffer_height { break };
    }
}
