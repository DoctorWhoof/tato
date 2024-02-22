use crate::{Color, Rect, Specs, Vec2};

/// Allows writing pixels to a frame buffer.
pub struct FrameBuf<S:Specs> 
where
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized,     //Tile count
{
    pub(super) pixels: [Color; S::RENDER_WIDTH * S::RENDER_HEIGHT],
    pub(super) viewport: Rect<i32>,
}

impl<S:Specs> FrameBuf<S>
where
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized,     //Tile count
{

    pub(super) fn new() -> Self {
        FrameBuf {
            pixels: [Color::default(); S::RENDER_WIDTH * S::RENDER_HEIGHT],
            viewport: Rect { x:0, y:0, w:S::RENDER_WIDTH as i32, h:S::RENDER_HEIGHT as i32 },
        }
    }


    pub fn width(&self) -> usize { S::RENDER_WIDTH }


    pub fn height(&self) -> usize { S::RENDER_HEIGHT }


    pub fn pixels(&self) -> &[Color; S::RENDER_WIDTH * S::RENDER_HEIGHT] { &self.pixels }


    pub fn viewport(&self) -> &Rect<i32> { &self.viewport }


    pub fn clear(&mut self, color:Color) {
        for pixel in self.pixels.iter_mut() {
            *pixel = color
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


    #[inline]
    pub fn draw_pixel(&mut self, x:usize, y:usize, color:Color){
        draw_pixel(&mut self.pixels, S::RENDER_WIDTH, x, y, color)
    }
    

    #[inline] #[allow(unused)]
    pub fn draw_line(&mut self, x0:i32, y0:i32, x1:i32, y1:i32, color:Color) {
        // TODO: Take viewport into account
        draw_line(&mut self.pixels, S::RENDER_WIDTH, x0, y0, x1, y1, color)
    }


    pub fn draw_rect(&mut self, rect:Rect<i32>, color:Color){
        // TODO: Take viewport into account
        let left = rect.x;
        let top = rect.y;
        let right = rect.x + rect.w - 1;
        let bottom = rect.y + rect.h - 1;
        if left > -1 && left < S::RENDER_WIDTH as i32 {
            draw_line(&mut self.pixels, S::RENDER_WIDTH, left, top, left, bottom, color)
        }
        if right > -1 && right < S::RENDER_WIDTH as i32 {
            draw_line(&mut self.pixels, S::RENDER_WIDTH,  right, top, right, bottom, color)
        }
        if top > -1 && top < S::RENDER_HEIGHT as i32 {
            draw_line(&mut self.pixels, S::RENDER_WIDTH,  left + 1, top, right - 1, top, color)
        }
        if bottom > -1 && bottom < S::RENDER_HEIGHT as i32 {
            draw_line(&mut self.pixels, S::RENDER_WIDTH,  left + 1, bottom, right - 1, bottom, color)
        }
    }


    pub fn draw_filled_rect(&mut self, rect:Rect<i32>, color:Color){
        // TODO: Take viewport into account
        let rect = {
            let x = i32::clamp(rect.x, 0, S::RENDER_WIDTH as i32 -1);
            let right = i32::clamp(rect.x + rect.w - 1, 0, S::RENDER_WIDTH as i32 - 1);
            let y = i32::clamp(rect.y, 0, S::RENDER_HEIGHT as i32 -1);
            let bottom = i32::clamp(rect.y + rect.h - 1, 0, S::RENDER_HEIGHT as i32 - 1);
            Rect { x, y, w: right-x, h: bottom - y }
        };
        for y in rect.y ..= rect.bottom() {
            for x in rect.x ..= rect.right() {
               self.draw_pixel(x as usize, y as usize, color)
            }
        }
    }

}



#[inline]
pub(crate) fn draw_pixel(pixels: &mut [Color], buffer_width:usize, x:usize, y:usize, color:Color){
    let index = (y * buffer_width) + x;
    if index > pixels.len() { return }
    pixels[index] = color;
}


pub(crate) fn draw_line(pixels: &mut [Color], buffer_width:usize, x0:i32, y0:i32, x1:i32, y1:i32, color:Color) {

    let buffer_height = pixels.len() / buffer_width;
        
    let mut x_head = (x0 as f32).clamp(0.0, buffer_width as f32);
    let mut y_head = (y0 as f32).clamp(0.0, buffer_height as f32);
    let x_tail = x1.clamp(0, buffer_width as i32);
    let y_tail = y1.clamp(0, buffer_height as i32);

    let w = x_tail - x_head as i32;
    let h = y_tail - y_head as i32;
    let longest = if w.abs() > h.abs() { w.abs() } else { h.abs() };
    let inc_x = w as f32 / longest as f32;
    let inc_y = h as f32 / longest as f32;

    for _ in 0 ..= longest as usize {
        draw_pixel(pixels, buffer_width, x_head as usize, y_head as usize, color);
        x_head += inc_x;
        y_head += inc_y;
        if x_head as usize >= buffer_width || y_head as usize >= buffer_height { break };
    }
}

