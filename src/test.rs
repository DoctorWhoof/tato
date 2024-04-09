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

    // for row in 0 .. tilemap.rows {
    //     for col in 0 ..tilemap.cols {
    //         let tile = tilemap.get_tile(col, row);
    //         if tile.is_collider(){
    //             print!("#")
    //         } else {
    //             print!(".")
    //         }
    //     }
    //     println!()
    // }

    let maybe_col = tilemap.raycast(1.0, 0.0, 8.0, 9.0);
    assert!(maybe_col.is_none());

    let maybe_col = tilemap.raycast(1.0, 0.0, 9.0, 9.0);
    assert!(maybe_col.is_some());

    // if let Some(col) = maybe_col{
    //     // println!("Collision at {:.1?}", col.pos);
    //     // assert_eq!(col.pos.x, 9.0)
    // };

    let maybe_col = tilemap.raycast(8.0, 5.0, -1.0, 0.0);
    assert!(maybe_col.is_some());

    // if let Some(col) = maybe_col{
    //     // println!("Collision at {:.1?}", col.pos);
    //     // assert_eq!(col.pos.x, 1.0)
    // };

}



#[test]
fn sweep_point_in_rect() {
    // let point = Vec2{x:10.0, y:5.0};
    // let point_vel = Vec2{x:5.0, y:-5.5};

    // let rect = Rect{x:20.0, y:0.0, w:10.0, h:10.0 };
    // let rect_vel = Vec2{x:-5.0, y:0.0};
    // let broad_rect = CollisionProbe::broad_rect(rect, rect_vel);
    
    // let result = CollisionProbe::broad_phase_point_in_rect(point, point_vel, rect, rect_vel);
    // println!("Collision result: {} for {:.1?} & {:.1?}", result, point + point_vel, rect + rect_vel);

    // let result = broad_rect.contains(point.x + point_vel.x, point.y + point_vel.y);
    // println!("Collision result: {} for {:.1?} & {:.1?}", result, point + point_vel, rect + rect_vel);

}



#[test]
fn ring_pool() {
    let mut pool = RingPool::<i32, 10>::new();

    // First 5 items
    for n in 0 .. 5 {
        let result = pool.push(n);
        assert!(result.is_none());
    }

    // Add 5 more without pushing any value out
    for n in 0 .. 5 {
        let result = pool.push(n + 5);
        assert!(result.is_none());
    }

    // Add 5 more, this time will push old values out
    for n in 0 .. 5 {
        let result = pool.push(n + 10);
        assert!(result.is_some());
        if let Some(value) = result {
            assert_eq!(value, n);
        }
    }

    // Pop 5 values, will reduce length
    for n in 0 .. 5 {
        let result = pool.pop();
        assert!(result.is_some());
        if let Some(value) = result {
            assert_eq!(value, n + 5);
        }
    }
    assert_eq!(pool.len(), 5);

    // Re-add 5 values, no values will be pushed out
    let len = pool.len();
    for n in 1 ..= 5 {
        let result = pool.push(100 * n);
        assert!(result.is_none());
        assert_eq!(pool.len(), len + n as usize);
    }




}