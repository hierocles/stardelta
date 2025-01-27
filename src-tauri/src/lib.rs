#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::command;
use std::fs;
use std::path::PathBuf;
use xdelta3::encode;
use xdelta3::decode;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;


#[derive(Debug, Serialize, Deserialize)]
struct CreatePatchArgs {
    original_file_path: String,
    edited_file_path: String,
    output_dir: String,
    original_file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApplyPatchArgs {
    file_to_patch_path: String,
    patch_file_path: String,
    output_dir: String,
    file_to_patch_name: String,
}

#[command]
fn create_patch(args: CreatePatchArgs) -> Result<(), String> {
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
    let output_path = PathBuf::from(&args.output_dir).join(format!("{}.xdelta", args.original_file_name));
    fs::write(&output_path, &patch).map_err(|e| {
        log::error!("Failed to write patch file: {}", e);
        e.to_string()
    })?;
    log::info!("Patch created successfully at {:?}", output_path);
    Ok(())
}

#[command]
fn apply_patch(args: ApplyPatchArgs) -> Result<(), String> {
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

pub fn run() {

    let builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_devtools::init()).plugin(tauri_plugin_devtools_app::init());

    builder
        .invoke_handler(tauri::generate_handler![create_patch, apply_patch])
        .plugin(tauri_plugin_decorum::init())
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
