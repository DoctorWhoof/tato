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

collision_layer_enum!{
    Layer {
        None,
        Hero,
        Enemies,
        Bullet
    }
}

tileset_enum!{
    TilesetID {
        Bg,
        Player,
        Enemies,
    }
}

palette_enum!{
    PaletteID {
        Bg,
        Fg
    }
}

group_enum!{
    GroupID {
        None,
        Wall,
        Brick
    }
}