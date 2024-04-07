use crate::*;


pub fn new_game() -> Game {
    // spud init
    let mut world = World::new(SPECS);
    let atlas = Atlas::<TilesetID, PaletteID>::load(SPECS, include_bytes!("../assets/converted/atlas"));

    world.renderer.load_palettes_from_atlas(&atlas);
    world.renderer.load_tileset(&atlas, TilesetID::Hud);
    world.renderer.load_tileset(&atlas, TilesetID::Bricks);
    world.renderer.load_tileset(&atlas, TilesetID::Bg);
    world.renderer.load_tileset(&atlas, TilesetID::Sprites);

    // BG
    let bg = world.entity_add(0);
    world.set_shape(bg, Shape::Bg { tileset_id: TilesetID::Bg.into(), tilemap_id: 0});
    world.collider_add(bg, Collider::from_tilemap(Layer::Wall));

    // Bricks
    let bricks = world.entity_add(1);
    world.set_shape(bricks, Shape::Bg { tileset_id: TilesetID::Bricks.into(), tilemap_id: 0 });
    world.collider_add(bricks, Collider::from_tilemap(Layer::Bricks));

    // Paddle
    let initial_paddle_pos = tato::Vec2 { x: 128.0, y: 160.0 };
    let paddle = Paddle {
        id: {
            let paddle = world.entity_add(1);
            world.set_shape(paddle, Shape::sprite_from_anim(TilesetID::Sprites, 0));
            world.collider_add(paddle, Collider::from_rect(Layer::Paddle, tato::Rect{x:-7.0, y:-7.0, w:15.0, h:15.0}));
            // world.collider_add(paddle, Collider::from_rect(Layer::Paddle, tato::Rect{x:-19.0, y:-12.0, w:38.0, h:24.0}), false);
            // world.collider_add(paddle, Collider::from_rect(Layer::Paddle, tato::Rect{x:-3.0, y:-3.0, w:7.0, h:7.0}), false);
            // world.collider_add(paddle, Collider::from_point(Layer::Paddle, 0.0, 0.0));
            world.enable_collision_with_layer(paddle, Layer::Bricks);
            world.enable_collision_with_layer(paddle, Layer::Wall);
            world.set_position(paddle, initial_paddle_pos);
            world.set_render_offset(paddle, -12,-8);
            paddle
        },
        vel: Vec2::zero(),
        initial_pos: initial_paddle_pos
    };

    // Puck
    let initial_puck_pos = tato::Vec2 { x: 160.0, y: 124.0 };
    let puck = Puck {
        id: {
            let puck = world.entity_add(1);
            world.set_shape(puck, Shape::sprite_from_anim(TilesetID::Sprites, 1));
            world.set_position(puck, initial_puck_pos);
            // TODO: Rect to tilemap collision is off, doesn't include the bottom corners?
            // world.collider_add(puck, Collider::from_rect(Layer::Puck, Rect{x:-2.0, y:-2.0, w:5.0, h:5.0}));
            world.collider_add(puck, Collider::from_point(Layer::Puck, 0.0, 1.0));
            world.enable_collision_with_layer(puck, Layer::Bricks);
            world.enable_collision_with_layer(puck, Layer::Wall);
            world.enable_collision_with_layer(puck, Layer::Paddle);
            world.set_render_offset(puck, -3, -4 );
            puck
        },
        vel: Vec2::zero(),
        initial_pos: initial_puck_pos,
    };

    // Return
    Game {
        world,
        atlas,
        paddle,
        puck,
        bricks,
        overlay: vec![]
    }
}