// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct BrowserState {
    current_url: String,
}

struct BrowserManager(Mutex<BrowserState>);

#[tauri::command]
async fn navigate_to(
    state: tauri::State<'_, BrowserManager>,
    url: String,
) -> Result<(), String> {
    let mut browser_state = state.0.lock().map_err(|e| e.to_string())?;
    browser_state.current_url = url.clone();
    
    // Here we'll add the Gecko navigation implementation
    Ok(())
}

fn main() {
    let browser_state = BrowserState {
        current_url: String::from("about:blank"),
    };

    tauri::Builder::default()
        .manage(BrowserManager(Mutex::new(browser_state)))
        .invoke_handler(tauri::generate_handler![navigate_to])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}