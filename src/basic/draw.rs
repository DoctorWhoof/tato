use libm::fabsf;
//Assumes the pixel buffer is indexed, 256 colors.

#[inline]
pub fn draw_pixel(pixels: &mut [u8], buffer_width:usize, x:usize, y:usize, color_index:u8){
    let index = (y as usize * buffer_width) + x as usize;
    pixels[index] = color_index;
}


pub fn draw_line(pixels: &mut [u8], buffer_width:usize, x0:i32, y0:i32, x1:i32, y1:i32, color_index:u8) {
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
        draw_pixel(pixels, buffer_width, x_head as usize, y_head as usize, color_index);
        x_head += inc_x;
        y_head += inc_y;
    }
}
