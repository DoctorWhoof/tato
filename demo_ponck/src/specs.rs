use tato::*;

pub const SPECS:Specs = Specs {
    render_width: 256,
    render_height: 192,
    atlas_width: 128,
    atlas_height: 128,
    tile_width: 8,
    tile_height: 8,
    colors_per_palette: 16,
};

tato::collision_layer_enum!{
    Layer {
        None,
        Puck,
        Paddle,
        Bricks,
        Wall
    }
}

tato::tileset_enum!{
    TilesetID {
        Hud,
        Sprites,
        Bricks,
        Bg,
        Title
    }
}

tato::palette_enum!{
    PaletteID {
        Msx,
    }
}

tato::group_enum!{
    GroupID {
        None,
        BrickSilver,
        BrickGold,
        BrickBlue,
        BrickRed,
        FloorGrille,
        FloorMetal,
        FloorGreen,
        WallBlue,
        WallGreen
    }
}

