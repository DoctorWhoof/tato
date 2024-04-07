use tato::*;
use crate::*;
    
pub fn new_game(specs:Specs) -> Game{

    // Spud init
    let mut world = World::<TilesetID, PaletteID>::new(specs);
    let atlas = Atlas::load( specs, include_bytes!("../assets/converted/atlas") );
    world.renderer.load_palettes_from_atlas(&atlas);  // TODO: get rid of this step... I always forget it!
    world.renderer.load_tileset(&atlas, TilesetID::Bg);
    world.renderer.load_tileset(&atlas, TilesetID::Player);
    world.renderer.load_tileset(&atlas, TilesetID::Enemies);


    // Sprites
    let player = Player {
        id: {
            let id = world.entity_add(2);
            let collider = Collider::new_rect_collider(Layer::Hero, Rect::new(-7.0, -7.0, 14.0, 14.0));

            world.set_shape(id, Shape::sprite_from_anim(TilesetID::Player, 0)); //TODO: T.O.C. look up
            world.set_position(id, Vec2::new(128.0, 160.0));
            world.set_render_offset(id, -8, -8);
            world.collider_add(id, collider, false);
            id
        },
        health: 10,
        score: 0,
        vel: Vec2::zero(),
    };

    let mut enemies = Grid::new(128.0, 48.0, 7, 3);
    for row in 0 ..enemies.size().y {
        for col in 0 ..enemies.size().x {
            enemies.set(col, row, {
                let human = world.entity_add(1);
                let collider = Collider::new_rect_collider(Layer::Enemies, Rect::new(-7.0, -7.0, 14.0, 14.0));

                world.set_shape(human, Shape::sprite_from_anim(TilesetID::Enemies, 0));
                world.set_render_offset(human, -8, -8);
                world.collider_add(human, collider, false);
                human
            });
        }
    }

    // Empty pool, will be populated as needed
    let bullets = RingPool::new();

    // Background
    let stars_bg_0 = world.entity_add(0);
    world.set_shape(stars_bg_0, Shape::Bg{ tileset_id: TilesetID::Bg.into(), tilemap_id: 0 });

    let stars_bg_1 = world.entity_add(0);
    world.set_shape(stars_bg_1, Shape::Bg{ tileset_id: TilesetID::Bg.into(), tilemap_id: 0 });
    world.set_position(stars_bg_1, Vec2{x:0.0, y:-216.0});

    let stars_fg_0 = world.entity_add(0);
    world.set_shape(stars_fg_0, Shape::Bg{ tileset_id: TilesetID::Bg.into(), tilemap_id: 1 });

    let stars_fg_1 = world.entity_add(0);
    world.set_shape(stars_fg_1, Shape::Bg{ tileset_id: TilesetID::Bg.into(), tilemap_id: 1 });
    world.set_position(stars_fg_1, Vec2{x:0.0, y:-216.0});

    
    // Return
    Game {
        world,
        player,
        enemies,
        bullets,
        stars_bg_0, stars_bg_1,
        stars_fg_0, stars_fg_1,
        // cooldown: 0.0
    }
}