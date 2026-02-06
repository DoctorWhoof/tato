// Flag bits
pub const FLAG_FLIP_H: u8 = 0b_1000_0000;
pub const FLAG_FLIP_V: u8 = 0b_0100_0000;
pub const FLAG_ROTATED: u8 = 0b_0010_0000;
pub const FLAG_FG: u8 = 0b_0001_0000;
pub const FLAG_COLLIDER: u8 = 0b_0000_1000;
pub const FLAG_TRIGGER: u8 = 0b_0000_0100;
pub const FLAG_INVISIBLE: u8 = 0b_0000_0010;

/// A single byte struct that stores a tile's render state such as
/// horizontal flip, vertical flip, rotation and custom data.
#[derive(Debug, Clone, Copy, PartialEq, Default, Hash)]
pub struct TileFlags(pub u8);

impl TileFlags {
    pub const fn new(flip_h: bool, flip_v: bool, collider: bool, trigger: bool) -> Self {
        let mut data: u8 = 0;

        if flip_h {
            data |= FLAG_FLIP_H;
        }
        if flip_v {
            data |= FLAG_FLIP_V;
        }
        if collider {
            data |= FLAG_COLLIDER;
        }
        if trigger {
            data |= FLAG_TRIGGER;
        }

        Self(data)
    }

    /// Consumes the original flag and ensures flip_x matches state
    pub const fn with_flip_x(self, state: bool) -> Self {
        if state { Self(self.0 | FLAG_FLIP_H) } else { Self(self.0 & !FLAG_FLIP_H) }
    }

    /// Consumes the original flag and ensures flip_y matches state
    pub const fn with_flip_y(self, state: bool) -> Self {
        if state { Self(self.0 | FLAG_FLIP_V) } else { Self(self.0 & !FLAG_FLIP_V) }
    }

    /// Consumes the original flag and ensures rotation matches state
    pub const fn with_rotation(self, state: bool) -> Self {
        if state { Self(self.0 | FLAG_ROTATED) } else { Self(self.0 & !FLAG_ROTATED) }
    }


    /// Consumes the original flag and ensures a BG tile is rendered in front of sprites.
    pub const fn with_fg(self, state:bool) -> Self {
        if state { Self(self.0 | FLAG_FG) } else { Self(self.0 & !FLAG_FG) }
    }

    /// Consumes the original flag and ensures tile is a collider.
    pub const fn with_collision(self, state:bool) -> Self {
        if state { Self(self.0 | FLAG_COLLIDER) } else { Self(self.0 & !FLAG_COLLIDER) }
    }

    /// Consumes the original flag and ensures tile is a trigger.
    pub const fn with_trigger(self, state:bool) -> Self {
        if state { Self(self.0 | FLAG_TRIGGER) } else { Self(self.0 & !FLAG_TRIGGER) }
    }

    /// Consumes original, sets desired transformations
    pub fn with_transform(mut self, flip_x: bool, flip_y: bool, rotation: bool) -> Self {
        self.set_flip_x(flip_x);
        self.set_flip_y(flip_y);
        self.set_rotation(rotation);
        self
    }

    pub const fn get_transform_bits(self) -> (bool, bool, bool) {
        (self.is_flipped_x(), self.is_flipped_y(), self.is_rotated())
    }

    /// If true flag will be flipped horizontally
    pub fn set_flip_x(&mut self, state: bool) {
        if state { self.0 |= FLAG_FLIP_H } else { self.0 &= !FLAG_FLIP_H }
    }

    /// If true flag will be flipped vertically
    pub fn set_flip_y(&mut self, state: bool) {
        if state { self.0 |= FLAG_FLIP_V } else { self.0 &= !FLAG_FLIP_V }
    }

    /// If true flag will be rotated 90 degrees.
    pub fn set_rotation(&mut self, state: bool) {
        if state { self.0 |= FLAG_ROTATED } else { self.0 &= !FLAG_ROTATED }
    }

    /// If true and this is a BG tile, it will be rendered in front of sprites.
    /// This value is ignored when used on sprites.
    pub fn set_fg(&mut self, state: bool) {
        if state { self.0 |= FLAG_FG } else { self.0 &= !FLAG_FG }
    }

    /// If true the tile will be a collider.
    pub fn set_collision(&mut self, state: bool) {
        if state { self.0 |= FLAG_COLLIDER } else { self.0 &= !FLAG_COLLIDER }
    }

    /// If true the tile will be a trigger.
    pub fn set_trigger(&mut self, state: bool) {
        if state { self.0 |= FLAG_TRIGGER } else { self.0 &= !FLAG_TRIGGER }
    }

    /// If false the tile will not render.
    pub fn set_invisible(&mut self, state: bool) {
        if state { self.0 |= FLAG_INVISIBLE } else { self.0 &= !FLAG_INVISIBLE }
    }

    pub fn rotate_up(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
    }

    pub fn rotate_left(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= FLAG_ROTATED; // set flags
    }

    pub fn rotate_down(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= FLAG_FLIP_V; // set flags
    }

    pub fn rotate_right(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= 0b_1010_0000; // set flags
    }

    /// Current horizontal flip state.
    pub const fn is_flipped_x(&self) -> bool {
        self.0 & FLAG_FLIP_H != 0
    }

    /// Current vertical flip state.
    pub const fn is_flipped_y(&self) -> bool {
        self.0 & FLAG_FLIP_V != 0
    }

    /// Current rotation state.
    pub const fn is_rotated(&self) -> bool {
        self.0 & FLAG_ROTATED != 0
    }

    /// If true and this is a BG tile, it will be rendered in front of sprites.
    /// This value is ignored when used on sprites.
    pub const fn is_fg(&self) -> bool {
        self.0 & FLAG_FG != 0
    }

    /// If true this tile is a collision tile.
    pub const fn is_collider(&self) -> bool {
        self.0 & FLAG_COLLIDER != 0
    }

    /// If true this tile is a trigger tile.
    pub const fn is_trigger(&self) -> bool {
        self.0 & FLAG_TRIGGER != 0
    }

    /// If true this tile is renderable.
    pub const fn is_invisible(&self) -> bool {
        self.0 & FLAG_INVISIBLE != 0
    }

    /// Toggles the horizontal flip state
    pub const fn toggle_flip_x(self) -> Self {
        Self(self.0 ^ FLAG_FLIP_H)
    }

    /// Toggles the vertical flip state
    pub const fn toggle_flip_y(self) -> Self {
        Self(self.0 ^ FLAG_FLIP_V)
    }

    /// Toggles the rotation state
    pub const fn toggle_rotation(self) -> Self {
        Self(self.0 ^ FLAG_ROTATED)
    }

    /// Toggles the foreground state (BG tiles only)
    pub const fn toggle_fg(self) -> Self {
        Self(self.0 ^ FLAG_FG)
    }

    /// Transform screen coordinates to tile coordinates based on flip/rotation flags.
    /// This is the canonical transformation used by the renderer.
    /// Given a screen position (x, y), returns which tile pixel (tx, ty) to read.
    #[inline]
    pub const fn transform_coords(self, x: u8, y: u8, size: u8) -> (u8, u8) {
        let high = size - 1;
        if self.is_rotated() {
            let rotated_x = high - y;
            let rotated_y = x;
            if self.is_flipped_x() {
                (rotated_x, high - rotated_y)
            } else if self.is_flipped_y() {
                (high - rotated_x, rotated_y)
            } else {
                (rotated_x, rotated_y)
            }
        } else {
            let tx = if self.is_flipped_x() { high - x } else { x };
            let ty = if self.is_flipped_y() { high - y } else { y };
            (tx, ty)
        }
    }
}
