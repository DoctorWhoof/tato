use tato_video::DynTilemap;

#[derive(Debug, Clone)]
pub struct TileCollision {
    pub col: i16,
    pub row: i16,
}


/// Tests tilemap collisions from start_col to end_col (inclusive). Returns
/// Some(TileCollision) if the tile is marked with the desired collision_group.
/// Can check in either direction - if end_col < start_col, checks in reverse.
pub fn line_to_tile_x(
    map: &impl DynTilemap,
    start_col: i16,
    end_col: i16,
    row: i16,
) -> Option<TileCollision> {
    if row < 0 || row >= map.rows() as i16 {
        return None;
    }
    let check_col = |col: i16| {
        (col >= 0)
            .then(|| map.get_cell(col, row))?
            .filter(|cell| cell.flags.is_collider())
            .map(|_| TileCollision { col, row })
    };
    if end_col < start_col {
        (end_col..=start_col).rev().find_map(check_col)
    } else {
        (start_col..=end_col).find_map(check_col)
    }
}

/// Tests tilemap collisions from start_row to end_row (inclusive). Returns
/// Some(TileCollision) if the tile is marked with the desired collision_group.
/// Can check in either direction - if end_row < start_row, checks in reverse.
pub fn line_to_tile_y(
    map: &impl DynTilemap,
    col: i16,
    start_row: i16,
    end_row: i16,
) -> Option<TileCollision> {
    if col < 0 || col >= map.columns() as i16 {
        return None;
    }
    let check_row = |row: i16| {
        (row >= 0)
            .then(|| map.get_cell(col, row))?
            .filter(|cell| cell.flags.is_collider())
            .map(|_| TileCollision { col, row })
    };
    if end_row < start_row {
        (end_row..=start_row).rev().find_map(check_row)
    } else {
        (start_row..=end_row).find_map(check_row)
    }
}
