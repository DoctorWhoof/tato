use crate::*;

#[derive(Debug, Clone, Default, Copy, PartialEq, Hash)]
pub struct Cell {
    pub id: TileID,
    pub flags: TileFlags,
    pub colors: Palette,
}

impl Cell {
    pub const fn new(id: u8, flags: u8, colors: u16) -> Self {
        Self {
            id: TileID(id),
            flags: TileFlags(flags),
            colors: Palette(colors),
        }
    }

    pub const fn with_id(self, id: u8) -> Self {
        Self { id: TileID(id), ..self }
    }

    pub const fn offset_id(self, offset: u8) -> Self {
        debug_assert!(offset as usize + (self.id.0 as usize) < 256, "Cell: offset causes overflow");
        Self { id: TileID(self.id.0 + offset), ..self }
    }

    pub const fn with_flags(self, flags: u8) -> Self {
        Self { flags: TileFlags(flags), ..self }
    }

    pub const fn with_colors(self, colors: Palette) -> Self {
        Self { colors, ..self }
    }

    pub const fn with_fg_flag(self) -> Self {
        Self { flags: self.flags.with_fg(), ..self }
    }

    pub const fn with_collision_flag(self) -> Self {
        Self { flags: self.flags.with_collision(), ..self }
    }

    pub const fn with_rotation_flag(self) -> Self {
        Self { flags: self.flags.with_rotation(), ..self }
    }

    pub const fn with_flip_x_flag(self) -> Self {
        Self { flags: self.flags.with_flip_x(), ..self }
    }

    pub const fn with_flip_y_flag(self) -> Self {
        Self { flags: self.flags.with_flip_y(), ..self }
    }
}
