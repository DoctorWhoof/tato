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
            let id = world.add_entity(2);
            world.set_shape(id, Shape::sprite_from_anim(TilesetID::Player, 0)); //TODO: T.O.C. look up
            world.set_position(id, Vec2::new(128.0, 160.0));
            world.set_render_offset(id, -8, -8);
            id
        },
        health: 10,
        score: 0,
        vel: Vec2::zero(),
    };

    let human = world.add_entity(1);
    world.set_shape(human, Shape::sprite_from_anim(TilesetID::Enemies, 0));
    world.set_position(human, Vec2::new(128.0, 32.0));
    world.set_render_offset(human, -8, -8);

    let bullets = RingPool::new();

    // Background
    let stars_bg_0 = world.add_entity(0);
    world.set_shape(stars_bg_0, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 0 });

    let stars_bg_1 = world.add_entity(0);
    world.set_shape(stars_bg_1, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 0 });
    world.set_position(stars_bg_1, Vec2{x:0.0, y:-192.0});

    let stars_fg_0 = world.add_entity(0);
    world.set_shape(stars_fg_0, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 1 });

    let stars_fg_1 = world.add_entity(0);
    world.set_shape(stars_fg_1, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 1 });
    world.set_position(stars_fg_1, Vec2{x:0.0, y:-192.0});

    // Return
    Game {
        world,
        atlas,
        player,
        bullets,
        stars_bg_0, stars_bg_1, stars_fg_0, stars_fg_1,
        cooldown: 0.0
    }
}