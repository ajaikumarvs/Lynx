// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lynx_lib::{AppState, BrowserState};

fn main() {
    let state = AppState(std::sync::Mutex::new(BrowserState::new()));

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            lynx_lib::create_tab,
            lynx_lib::close_tab,
            lynx_lib::navigate_to,
            lynx_lib::go_back,
            lynx_lib::go_forward,
            lynx_lib::refresh,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}