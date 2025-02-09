#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod swf;
mod xdelta;
mod ba2;

use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    let builder = builder
        .plugin(tauri_plugin_devtools::init())
        .plugin(tauri_plugin_devtools_app::init());

    builder
        .invoke_handler(tauri::generate_handler![
            xdelta::create_patch,
            xdelta::apply_patch,
            swf::convert_swf_to_json,
            swf::convert_json_to_swf,
            swf::apply_json_modifications,
            swf::get_file_size,
            swf::batch_process_swf,
            swf::read_file_to_string
        ])
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
