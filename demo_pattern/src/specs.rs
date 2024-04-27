use tato::*;

pub const SPECS:Specs = Specs {
    render_width: 256,
    render_height: 192,
    atlas_width: 128,
    atlas_height: 128,
    tile_width: 8,
    tile_height: 8,
    colors_per_palette: 16,
    max_loaded_anims: 4,
    max_loaded_fonts: 1,
    max_loaded_tilemaps: 1,
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
    }
}

palette_enum!{
    PaletteID {
        Bg,
    }
}

group_enum!{
    GroupID {
        None,
    }
}