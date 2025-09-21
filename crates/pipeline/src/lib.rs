mod builders;

use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

pub use builders::*;

mod code_gen;
pub(crate) use code_gen::*;

mod palette_image;
pub(crate) use palette_image::*;

// Initialization tracking
static BUILD_INITIALIZED: AtomicBool = AtomicBool::new(false);
static FORCE_REPROCESS: AtomicBool = AtomicBool::new(false);
static ASSET_IMPORT_PATH: OnceLock<String> = OnceLock::new();

fn ensure_init_build() {
    if !BUILD_INITIALIZED.load(Ordering::Relaxed) {
        panic!(
            "\n\x1b[31m\nERROR: init_build() must be called before using any pipeline builders. Add init_build() to your build.rs file.\n\x1b[0m\n"
        );
    }
}

#[cfg(test)]
pub fn reset_init_build() {
    BUILD_INITIALIZED.store(false, Ordering::Relaxed);
}

// Public API
pub use {GroupBuilder, PaletteBuilder, TilesetBuilder};

pub fn should_regenerate_file(file_path: &str) -> bool {
    // If force reprocess is enabled, always regenerate
    if FORCE_REPROCESS.load(Ordering::Relaxed) {
        return true;
    }

    let metadata = load_metadata();

    if let Some(current_timestamp) = get_file_timestamp(file_path) {
        if let Some(&stored_timestamp) = metadata.get(file_path) {
            return current_timestamp > stored_timestamp;
        }
        // File not in metadata, should regenerate
        return true;
    }

    // File doesn't exist or can't read timestamp, should regenerate
    true
}

pub fn mark_file_processed(file_path: &str) {
    let mut metadata = load_metadata();

    if let Some(timestamp) = get_file_timestamp(file_path) {
        metadata.insert(file_path.to_string(), timestamp);
        save_metadata(&metadata);
    }
}

/// Initializes build script integration with cargo
pub fn init_build(asset_import_path: &str, force_reprocess: bool) {
    // Store asset import path globally
    let _ = ASSET_IMPORT_PATH.set(String::from(asset_import_path));

    // Store force reprocess flag
    FORCE_REPROCESS.store(force_reprocess, Ordering::Relaxed);

    // Mark initialization as complete
    BUILD_INITIALIZED.store(true, Ordering::Relaxed);

    // Cargo build setup
    println!("cargo:warning=Working Dir:{:?}", std::env::current_dir().ok().unwrap());
    println!("cargo:warning=Asset import path: {}", asset_import_path);
    println!("cargo:warning=Force reprocess: {}", force_reprocess);

    // Only watch build.rs - we'll handle file change detection manually
    println!("cargo:rerun-if-changed=build.rs");
}

pub fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}

// Metadata file handling
fn get_asset_import_path() -> String {
    ASSET_IMPORT_PATH.get().expect("init_build must be called before using asset functions").clone()
}

fn get_metadata_path() -> String {
    format!("{}/.tato_metadata", get_asset_import_path())
}

fn load_metadata() -> HashMap<String, u64> {
    let metadata_path = get_metadata_path();
    let mut metadata = HashMap::new();
    let mut needs_cleanup = false;

    if let Ok(content) = read_to_string(&metadata_path) {
        for line in content.lines() {
            if let Some((path, timestamp)) = line.split_once('=') {
                if let Ok(ts) = timestamp.parse::<u64>() {
                    // Check if file still exists (lazy cleanup)
                    if std::fs::metadata(path).is_ok() {
                        metadata.insert(path.to_string(), ts);
                    } else {
                        needs_cleanup = true;
                    }
                }
            }
        }

        // Save cleaned metadata if we removed any entries
        if needs_cleanup {
            save_metadata(&metadata);
        }
    }

    metadata
}

fn save_metadata(metadata: &HashMap<String, u64>) {
    let metadata_path = get_metadata_path();

    if let Ok(mut file) = File::create(&metadata_path) {
        for (path, timestamp) in metadata {
            let _ = writeln!(file, "{}={}", path, timestamp);
        }
    }
}

fn get_file_timestamp(path: &str) -> Option<u64> {
    std::fs::metadata(path).ok()?.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs().into()
}
