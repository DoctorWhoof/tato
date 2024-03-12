use tato::{Specs, EnumID};

pub const SPECS:Specs = Specs {
    render_width: 256,
    render_height: 192,
    atlas_width: 128,
    atlas_height: 128,
    tile_width: 8,
    tile_height: 8,
    colors_per_palette: 16,
};

tato::enum_id!{
    TilesetID {
        Hud,
        Sprites,
        Bg,
    }
}

tato::enum_id!{
    PaletteID {
        Main
    }
}

tato::enum_id!{
    GroupID {
        None,
        Wall,
        Brick
    }
}

