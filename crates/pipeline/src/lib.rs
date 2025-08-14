mod builders;
pub use builders::*;

mod code_gen;
pub(crate) use code_gen::*;

mod palette_image;
pub(crate) use palette_image::*;

// Public API
pub use {PaletteBuilder, TilesetBuilder, GroupBuilder};

/// Initializes build script integration with cargo
pub fn init_build() {
    // Cargo build setup
    println!("cargo:warning=Running Build Script!");
    println!("cargo:warning=Working Dir:{:?}", std::env::current_dir().ok().unwrap());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/*.*");
}

pub fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}
