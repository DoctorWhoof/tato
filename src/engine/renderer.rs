use crate::{Color, Rect, Specs, Vec2, Tileset};
use libm::fabsf;
use slotmap::{new_key_type, SlotMap};

new_key_type! { struct TilesetKey; }

/// Allows writing pixels to a frame buffer.
pub struct Renderer<S:Specs> 
where
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized,     //Tile count
{
    pub(super) pixels: [Color; S::RENDER_WIDTH * S::RENDER_HEIGHT],
    pub(super) viewport: Rect<i32>,
    // pub(super) tilesets: SlotMap<TilesetKey, Tileset>,
    // pub(crate) rects:[Rect<u8>; (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)],
}

impl<S:Specs> Renderer<S>
where
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized,     //Tile count
{

    pub(super) fn new() -> Self {
        Renderer {
            pixels: [Color::default(); S::RENDER_WIDTH * S::RENDER_HEIGHT],
            viewport: Rect { x:0, y:0, w:S::RENDER_WIDTH as i32, h:S::RENDER_HEIGHT as i32 },
            // rects: core::array::from_fn( |i| {
            //     // generates all tiles
            //     let tile_x = i * S::TILE_WIDTH as usize;
            //     let x = (tile_x % S::ATLAS_WIDTH) as u8;
            //     let y = ((tile_x / S::ATLAS_WIDTH) * S::TILE_HEIGHT as usize) as u8;
            //     Rect{
            //         x,
            //         y,
            //         w:S::TILE_WIDTH,
            //         h:S::TILE_HEIGHT
            //     }
            // }),
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
    pub(super) fn draw_pixel(&mut self, x:usize, y:usize, color:Color){
        draw_pixel(&mut self.pixels, S::RENDER_WIDTH, x, y, color)
    }
    

    #[inline] #[allow(unused)]
    pub(super) fn draw_line(&mut self, x0:i32, y0:i32, x1:i32, y1:i32, color:Color) {
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
pub fn draw_pixel(pixels: &mut [Color], buffer_width:usize, x:usize, y:usize, color:Color){
    let index = (y * buffer_width) + x;
    pixels[index] = color;
}


pub fn draw_line(pixels: &mut [Color], buffer_width:usize, x0:i32, y0:i32, x1:i32, y1:i32, color:Color) {
    let buffer_height = pixels.len() / buffer_width;
    let x_head = i32::max(x0, 0);
    let mut x_head = i32::min(x_head, (buffer_width-1) as i32) as f32;

    let y_head = i32::max(y0, 0);
    let mut y_head = i32::min(y_head, (buffer_height-1) as i32) as f32;

    let x_tail = i32::max(x1, 0);
    let x_tail = i32::min(x_tail, (buffer_width-1) as i32) as f32;

    let y_tail = i32::max(y1, 0);
    let y_tail = i32::min(y_tail, (buffer_height-1) as i32) as f32;

    let w = fabsf(x_tail - x_head);
    let h = fabsf(y_tail - y_head);
    let longest = if w > h { w } else { h };
    let inc_x = w / longest;
    let inc_y = h / longest;

    for _ in 0 ..= longest as usize {
        draw_pixel(pixels, buffer_width, x_head as usize, y_head as usize, color);
        x_head += inc_x;
        y_head += inc_y;
    }
}

