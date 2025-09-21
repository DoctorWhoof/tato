mod builders;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

pub use builders::*;

mod code_gen;
pub(crate) use code_gen::*;

mod palette_image;
pub(crate) use palette_image::*;

// Initialization tracking
static INIT_BUILD_CALLED: AtomicBool = AtomicBool::new(false);

fn ensure_init_build() {
    if !INIT_BUILD_CALLED.load(Ordering::Relaxed) {
        panic!("\n\x1b[31m\nERROR: init_build() must be called before using any pipeline builders. Add init_build() to your build.rs file.\n\x1b[0m\n");
    }
}

#[cfg(test)]
pub fn reset_init_build() {
    INIT_BUILD_CALLED.store(false, Ordering::Relaxed);
}

// Public API
pub use {GroupBuilder, PaletteBuilder, TilesetBuilder};

fn watch_dir<P: AsRef<Path>>(dir: P) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                watch_dir(&path); // recurse into subdir
            } else if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}

/// Initializes build script integration with cargo
pub fn init_build(asset_import_path: &str) {
    // Mark initialization as complete
    INIT_BUILD_CALLED.store(true, Ordering::Relaxed);

    // Cargo build setup
    println!("cargo:warning=Working Dir:{:?}", std::env::current_dir().ok().unwrap());
    println!("cargo:warning=Running Build Script if changes are detected...");
    // rerun is build script changes
    println!("cargo:rerun-if-changed=build.rs");
    // rerun if anything changes under asset import path
    watch_dir(asset_import_path);
}

pub fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}
