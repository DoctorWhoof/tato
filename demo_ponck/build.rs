#[path ="src/specs.rs"] mod specs;
use specs::*;
use tato_pipe::*;


fn main() {
    let mut atlas = AtlasBuilder::<TilesetID, PaletteID, GroupID>::new(SPECS);
    println!("cargo:warning=Running Build Script!");
    
    atlas.init_tileset( TilesetID::Hud, PaletteID::Msx );
    atlas.init_font("assets/font.png", TilesetID::Hud, GroupID::None);

    atlas.init_tileset( TilesetID::Sprites, PaletteID::Msx );
    atlas.init_anim("assets/sprite_paddle.png", 10, 3, 2, TilesetID::Sprites, GroupID::None);
    atlas.init_anim("assets/sprite_puck.png", 10, 1, 1, TilesetID::Sprites, GroupID::None);
    
    
    atlas.init_tileset( TilesetID::Bricks, PaletteID::Msx );
    atlas.init_group("assets/groups/brick_empty.png", TilesetID::Bricks, GroupID::None, false);

    atlas.init_group("assets/groups/brick_silver.png", TilesetID::Bricks, GroupID::BrickSilver, true);
    atlas.init_group("assets/groups/brick_gold.png", TilesetID::Bricks, GroupID::BrickGold, true);
    atlas.init_group("assets/groups/brick_blue.png", TilesetID::Bricks, GroupID::BrickBlue, true);
    atlas.init_group("assets/groups/brick_red.png", TilesetID::Bricks, GroupID::BrickRed, true);

    atlas.init_group("assets/groups/brick_silver_side.png", TilesetID::Bricks, GroupID::BrickSilver, false);
    atlas.init_group("assets/groups/brick_gold_side.png", TilesetID::Bricks, GroupID::BrickGold, false);
    atlas.init_group("assets/groups/brick_red_side.png", TilesetID::Bricks, GroupID::BrickRed, false);
    atlas.init_group("assets/groups/brick_blue_side.png", TilesetID::Bricks, GroupID::BrickBlue, false);

    atlas.init_tilemap("assets/level_00.png", TilesetID::Bricks, GroupID::None); // tilemap 0
    

    atlas.init_tileset( TilesetID::Bg, PaletteID::Msx );
    atlas.init_group("assets/groups/floor_grille.png", TilesetID::Bg, GroupID::FloorGrille, false);
    atlas.init_group("assets/groups/floor_green.png", TilesetID::Bg, GroupID::FloorGreen, false);
    atlas.init_group("assets/groups/floor_metal.png", TilesetID::Bg, GroupID::FloorMetal, false);

    atlas.init_group("assets/groups/wall_blue.png", TilesetID::Bg, GroupID::WallBlue, true);
    atlas.init_group("assets/groups/wall_green.png", TilesetID::Bg, GroupID::WallGreen, true);

    atlas.init_tilemap("assets/bg.png", TilesetID::Bg, GroupID::None); // tilemap 0

    atlas.init_tileset( TilesetID::Title, PaletteID::Msx );
    atlas.init_tilemap("assets/title.png", TilesetID::Title, GroupID::None); // tilemap 0
    
    atlas.save("assets/converted/atlas");
}