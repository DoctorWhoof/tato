use core::f32::consts::PI;
use libm::roundf;

pub const DEG_TO_RAD:f32 = PI / 180.0;
pub const RAD_TO_DEG:f32 = 180.0 / PI;


pub fn lerp(a:f32, b:f32, t:f32) -> f32 {
    a + ((b-a) * t)
}


pub fn quantize(value: f32, size: f32) -> f32 {
    roundf(value/size) * size
}


pub fn mirror_angle(angle: f32, mirror_normal: f32) -> f32 {
    2.0 * mirror_normal - angle
}


pub fn invert_angle(angle: f32) -> f32 {
    let inverted_angle = angle + core::f32::consts::PI;
    if inverted_angle > core::f32::consts::PI {
        inverted_angle - 2.0 * core::f32::consts::PI
    } else {
        inverted_angle
    }
}
