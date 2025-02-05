use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;
use xdelta3::{decode, encode};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePatchArgs {
    pub original_file_path: String,
    pub edited_file_path: String,
    pub output_dir: String,
    pub original_file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyPatchArgs {
    pub file_to_patch_path: String,
    pub patch_file_path: String,
    pub output_dir: String,
    pub file_to_patch_name: String,
}

#[command]
pub fn create_patch(args: CreatePatchArgs) -> Result<(), String> {
    log::trace!("Creating patch with args: {:?}", args);
    let original = fs::read(&args.original_file_path).map_err(|e| {
        log::error!("Failed to read original file: {}", e);
        e.to_string()
    })?;
    let edited = fs::read(&args.edited_file_path).map_err(|e| {
        log::error!("Failed to read edited file: {}", e);
        e.to_string()
    })?;
    let patch = encode(&edited, &original).ok_or_else(|| {
        let msg = "Encoding failed".to_string();
        log::error!("{}", msg);
        msg
    })?;
    let output_path =
        PathBuf::from(&args.output_dir).join(format!("{}.xdelta", args.original_file_name));
    fs::write(&output_path, &patch).map_err(|e| {
        log::error!("Failed to write patch file: {}", e);
        e.to_string()
    })?;
    log::info!("Patch created successfully at {:?}", output_path);
    Ok(())
}

#[command]
pub fn apply_patch(args: ApplyPatchArgs) -> Result<(), String> {
    log::trace!("Applying patch with args: {:?}", args);
    let file_to_patch = fs::read(&args.file_to_patch_path).map_err(|e| {
        log::error!("Failed to read file to patch: {}", e);
        e.to_string()
    })?;
    let patch = fs::read(&args.patch_file_path).map_err(|e| {
        log::error!("Failed to read patch file: {}", e);
        e.to_string()
    })?;
    let decoded = decode(&patch, &file_to_patch).ok_or_else(|| {
        let msg = "Decoding failed".to_string();
        log::error!("{}", msg);
        msg
    })?;
    let output_path = PathBuf::from(&args.output_dir).join(&args.file_to_patch_name);
    fs::write(&output_path, &decoded).map_err(|e| {
        log::error!("Failed to write patched file: {}", e);
        e.to_string()
    })?;
    log::info!("Patch applied successfully at {:?}", output_path);
    Ok(())
}
