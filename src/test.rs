#[cfg(test)]
use crate::*;

#[test]
fn tilemap() {
    let tilemap = Tilemap {
        cols: 10,
        rows: 10,
        tiles: core::array::from_fn(|i| {
            let mut tile = Tile::default();
            if i % 10 == 9 || i % 10 == 0 {
                tile.set_collider(true)
            }
            tile
        }),
        id: 0,
        tileset: 0,
        palette: 0,
        bg_buffers: Default::default(),
    };

    for row in 0 .. tilemap.rows {
        for col in 0 ..tilemap.cols {
            let tile = tilemap.get_tile(col, row);
            if tile.is_collider(){
                print!("#")
            } else {
                print!(".")
            }
        }
        println!()
    }
    let maybe_col = tilemap.raycast(1.0, 0.0, 8.0, 9.0);
    assert!(maybe_col.is_none());
    let maybe_col = tilemap.raycast(1.0, 0.0, 9.0, 9.0);
    assert!(maybe_col.is_some());

    if let Some(col) = maybe_col{
        println!("Collision at {:.1?}", col.point);
        assert_eq!(col.point.x, 9.0)
    };

    let maybe_col = tilemap.raycast(8.0, 5.0, -1.0, 0.0);
    assert!(maybe_col.is_some());

    if let Some(col) = maybe_col{
        println!("Collision at {:.1?}", col.point);
        assert_eq!(col.point.x, 1.0)
    };

}
