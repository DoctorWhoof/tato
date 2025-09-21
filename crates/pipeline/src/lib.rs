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

#[derive(Clone)]
pub struct BuildSettings {
    pub asset_import_path: String,
    pub force_reprocess: bool,
}

// Initialization tracking
static INIT_BUILD_CALLED: AtomicBool = AtomicBool::new(false);
static BUILD_SETTINGS: OnceLock<BuildSettings> = OnceLock::new();

fn get_build_settings() -> BuildSettings {
    BUILD_SETTINGS.get()
        .expect("init_build must be called before using build settings")
        .clone()
}

fn ensure_init_build() {
    if !INIT_BUILD_CALLED.load(Ordering::Relaxed) {
        panic!(
            "\n\x1b[31m\nERROR: init_build() must be called before using any pipeline builders. Add init_build() to your build.rs file.\n\x1b[0m\n"
        );
    }
}

#[cfg(test)]
pub fn reset_init_build() {
    INIT_BUILD_CALLED.store(false, Ordering::Relaxed);
}

// Public API
pub use {GroupBuilder, PaletteBuilder, TilesetBuilder};

pub fn should_regenerate_file(file_path: &str) -> bool {
    // If force reprocess is enabled, always regenerate
    if get_build_settings().force_reprocess {
        println!("cargo:warning=Force reprocess enabled for: {}", file_path);
        return true;
    }

    let metadata = load_metadata();

    if let Some(current_timestamp) = get_file_timestamp(file_path) {
        if let Some(&stored_timestamp) = metadata.get(file_path) {
            let should_regen = current_timestamp > stored_timestamp;
            if should_regen {
                println!("cargo:warning=File changed - will regenerate: {}", file_path);
            }
            return should_regen;
        }
        // File not in metadata, should regenerate
        println!("cargo:warning=New file detected: {}", file_path);
        return true;
    }

    // File doesn't exist or can't read timestamp, should regenerate
    println!("cargo:warning=Cannot read timestamp for: {}", file_path);
    true
}

pub fn mark_file_processed(file_path: &str) {
    let mut metadata = load_metadata();

    if let Some(timestamp) = get_file_timestamp(file_path) {
        println!("cargo:warning=Marking file as processed: {} with timestamp: {}", file_path, timestamp);
        metadata.insert(file_path.to_string(), timestamp);
        save_metadata(&metadata);
    }
}

/// Initializes build script integration with cargo
pub fn init_build(settings: BuildSettings) {
    // Store build settings globally
    let _ = BUILD_SETTINGS.set(settings.clone());

    // Mark initialization as complete
    INIT_BUILD_CALLED.store(true, Ordering::Relaxed);

    // Cargo build setup
    println!("cargo:warning=Working Dir:{:?}", std::env::current_dir().ok().unwrap());
    println!("cargo:warning=Asset import path: {}", settings.asset_import_path);
    println!("cargo:warning=Force reprocess: {}", settings.force_reprocess);

    // Watch build.rs
    println!("cargo:rerun-if-changed=build.rs");

    // Watch all asset files so cargo will rerun when they change
    watch_asset_files(&settings.asset_import_path);
}

fn watch_asset_files(asset_import_path: &str) {
    use std::path::Path;

    let asset_path = Path::new(asset_import_path);
    if let Ok(entries) = std::fs::read_dir(asset_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Only watch actual asset files, not system files
                    if is_asset_file(file_name) {
                        println!("cargo:rerun-if-changed={}", path.display());
                    }
                }
            } else if path.is_dir() {
                // Recursively watch subdirectories
                if let Some(path_str) = path.to_str() {
                    watch_asset_files(path_str);
                }
            }
        }
    }
}

fn is_asset_file(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") ||
    lower.ends_with(".bmp") || lower.ends_with(".gif") || lower.ends_with(".tga")
}

pub fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}

// Metadata file handling
fn get_asset_import_path() -> String {
    get_build_settings().asset_import_path
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
