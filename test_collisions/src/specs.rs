use tato::*;
// use tato::strum::EnumCount;
// use tato::strum_macros::EnumCount as EnumCountMacro;

// pub const SPECS:Specs = Specs {
//     render_width: 256,
//     render_height: 192,
//     atlas_width: 128,
//     atlas_height: 128,
//     tile_width: 8,
//     tile_height: 8,
//     colors_per_palette: 16,
// };

enum_id!{
    TilesetID {
        Default
    }
}
    

enum_id!{
    PaletteID {
        Default
    }
}


enum_id!{
    GroupID {
        None,
        Wall,
        Brick
    }
}




