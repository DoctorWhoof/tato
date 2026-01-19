mod builders;

use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::sync::{OnceLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

pub use builders::*;

mod code_gen;
pub(crate) use code_gen::*;
pub use code_gen::{format_cell_compact, format_tile_compact};

mod palette_image;
pub(crate) use palette_image::*;

pub use {BankBuilder, GroupBuilder, PaletteBuilder};

/// Build pipeline configuration.
#[derive(Clone)]
pub struct BuildSettings {
    /// Directory containing input asset files.
    pub asset_import_path: String,

    /// Directory where generated Rust files will be written. If set to "src", mod.rs generation is skipped.
    pub asset_export_path: String,

    /// Clears the export directory before generating files. Dangerous paths are blocked.
    pub clear_export_path: bool,

    /// Forces regeneration of all files regardless of timestamps.
    pub force_reprocess: bool,
}

// Initialization tracking
static INIT_BUILD_CALLED: AtomicBool = AtomicBool::new(false);
static BUILD_SETTINGS: OnceLock<BuildSettings> = OnceLock::new();
static GENERATED_FILES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Initializes the build pipeline. Call this first in your `build.rs`.
pub fn init_build(settings: BuildSettings) {
    // Store build settings globally
    let _ = BUILD_SETTINGS.set(settings.clone());

    // Mark initialization as complete
    INIT_BUILD_CALLED.store(true, Ordering::Relaxed);

    // Clear generated files list from any previous runs
    GENERATED_FILES.lock().unwrap().clear();

    // Cargo build setup
    println!("cargo:warning=Working Dir:{:?}", std::env::current_dir().ok().unwrap());
    println!("cargo:warning=Asset import path: {}", settings.asset_import_path);
    println!("cargo:warning=Asset export path: {}", settings.asset_export_path);
    println!("cargo:warning=Clear export path: {}", settings.clear_export_path);
    println!("cargo:warning=Force reprocess: {}", settings.force_reprocess);

    // Clear export path if requested
    if settings.clear_export_path {
        clear_export_path_safely(&settings.asset_export_path);
    }

    // Watch build.rs
    println!("cargo:rerun-if-changed=build.rs");

    // Watch all asset files so cargo will rerun when they change
    watch_asset_files(&settings.asset_import_path);
}

/// Safely clears the export path with multiple safeguards
fn clear_export_path_safely(export_path: &str) {
    use std::path::Path;

    // Safeguard: Don't allow empty paths
    if export_path.is_empty() {
        panic!("ERROR: Export path cannot be empty");
    }

    // Safeguard: Don't allow parent directory traversal
    if export_path.contains("..") {
        panic!("ERROR: Export path cannot contain '..' (parent directory traversal): {}", export_path);
    }

    // Safeguard: Don't allow absolute paths (must be relative to working dir)
    let path = Path::new(export_path);
    if path.is_absolute() {
        panic!("ERROR: Export path must be relative, not absolute: {}", export_path);
    }

    // Normalize the path for checking
    let normalized = export_path.trim_start_matches("./").trim_end_matches("/");

    // These top-level paths should never be cleared (but can be used as export paths)
    let no_clear_paths = [".", "src", "/", "target", "cargo", ".git", ".cargo"];
    if no_clear_paths.contains(&normalized) {
        println!(
            "cargo:warning=Skipping clear for '{}' (top-level directory). Files will be overwritten in place.",
            export_path
        );
        // Still ensure the directory exists
        if !path.exists() {
            if let Err(e) = std::fs::create_dir_all(path) {
                println!("cargo:warning=Failed to create export directory: {}", e);
            }
        }
        return;
    }

    // If all checks pass, proceed with clearing
    if path.exists() {
        match std::fs::remove_dir_all(path) {
            Ok(_) => {
                println!("cargo:warning=Cleared export directory: {}", export_path);
                // Recreate the directory
                if let Err(e) = std::fs::create_dir_all(path) {
                    println!("cargo:warning=Failed to recreate export directory: {}", e);
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to clear export directory {}: {}", export_path, e);
            }
        }
    } else {
        // Directory doesn't exist, create it
        if let Err(e) = std::fs::create_dir_all(path) {
            println!("cargo:warning=Failed to create export directory: {}", e);
        } else {
            println!("cargo:warning=Created export directory: {}", export_path);
        }
    }
}

/// Registers a generated file for inclusion in mod.rs
pub(crate) fn register_generated_file(file_path: &str) {
    use std::path::Path;

    let settings = get_build_settings();
    let export_path = &settings.asset_export_path;

    // Normalize paths by removing leading "./" and trailing "/"
    let normalized_export = export_path.trim_start_matches("./").trim_end_matches("/");
    let normalized_file = file_path.trim_start_matches("./");

    // Extract the module name from the file path
    let path = Path::new(normalized_file);

    // Try to get the file stem (filename without extension)
    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
        // Check if the file is in the export directory
        if normalized_file.starts_with(normalized_export) {
            // Don't add if it's already in the list
            let mut files = GENERATED_FILES.lock().unwrap();
            if !files.contains(&file_stem.to_string()) {
                files.push(file_stem.to_string());
            }
        }
    }
}

/// Writes a mod.rs file that exports all generated modules
fn write_mod_file(export_path: &str) {
    use std::path::Path;

    // Don't create mod.rs if exporting directly to "src"
    let normalized = export_path.trim_start_matches("./").trim_end_matches("/");
    if normalized == "src" {
        return;
    }

    let files = GENERATED_FILES.lock().unwrap();
    if files.is_empty() {
        return;
    }

    let mod_path = Path::new(export_path).join("mod.rs");

    match File::create(&mod_path) {
        Ok(mut file) => {
            // Write header
            let _ = writeln!(file, "// Auto-generated mod.rs - exports all generated modules");
            let _ = writeln!(file, "");

            // Write pub mod declarations for each generated file
            for module_name in files.iter() {
                let _ = writeln!(file, "pub mod {};", module_name);
            }

            let _ = writeln!(file, "");

            // Write pub use statements to re-export all module contents
            for module_name in files.iter() {
                let _ = writeln!(file, "pub use {}::*;", module_name);
            }

            println!("cargo:warning=Created mod.rs with {} modules", files.len());
        }
        Err(e) => {
            println!("cargo:warning=Failed to create mod.rs: {}", e);
        }
    }
}

/// Generates a `mod.rs` file that declares and re-exports all non-empty generated modules.
/// Call this at the end of your `build.rs` after all `write()` calls.
pub fn finalize_build() {
    let settings = get_build_settings();
    write_mod_file(&settings.asset_export_path);
}

pub(crate) fn get_build_settings() -> BuildSettings {
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

pub(crate) fn should_regenerate_file(file_path: &str) -> bool {
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

pub(crate) fn mark_file_processed(file_path: &str) {
    let mut metadata = load_metadata();

    if let Some(timestamp) = get_file_timestamp(file_path) {
        println!("cargo:warning=Marking file as processed: {} with timestamp: {}", file_path, timestamp);
        metadata.insert(file_path.to_string(), timestamp);
        save_metadata(&metadata);
    }
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

pub(crate) fn strip_path_name(path: &str) -> String {
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
