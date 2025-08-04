use tato_video::{DynTilemap, Tilemap};

pub struct TileCollision {
    pub col: i16,
    pub row: i16,
}

pub fn line_to_tile_y<const LEN: usize>(
    map: &Tilemap<LEN>,
    col: i16,
    start_row: i16,
    delta: i16,
    collision_group: u16, // All collider tiles must belong to this group
) -> Option<TileCollision> {
    if col < 0 || col >= map.columns as i16 {
        return None;
    }

    for row in start_row..start_row + delta {
        if row < 0 {
            continue;
        }
        let Some(cell) = map.get_cell(col as u16, row as u16) else {
            continue;
        };

        if cell.group & collision_group != 0 {
            return Some(TileCollision { col, row });
        }
    }
    None
}

pub fn line_to_tile_x<const LEN: usize>(
    map: &Tilemap<LEN>,
    row: i16,
    start_col: i16,
    delta: i16,
    collision_group: u16, // All collider tiles must belong to this group
) -> Option<TileCollision> {
    if row < 0 || row >= map.rows as i16 {
        return None;
    }

    for col in start_col..start_col + delta {
        if col < 0 {
            continue;
        }
        let Some(cell) = map.get_cell(col as u16, row as u16) else {
            continue;
        };

        if cell.group & collision_group != 0 {
            return Some(TileCollision { col, row });
        }
    }
    None
}
