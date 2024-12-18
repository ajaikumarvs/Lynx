// src-tauri/src/lib.rs
use tauri::{Runtime, State, Manager, Window};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use webkit2gtk::{WebView, WebContext, WebViewExt, LoadEvent};
use gtk::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tab {
    id: String,
    url: String,
    title: String,
    can_go_back: bool,
    can_go_forward: bool,
}

#[derive(Debug, Serialize)]
pub struct TabUpdateEvent {
    tab_id: String,
    title: String,
    url: String,
    can_go_back: bool,
    can_go_forward: bool,
}

pub struct BrowserState {
    tabs: Vec<Tab>,
    active_tab: Option<String>,
    webviews: std::collections::HashMap<String, WebView>,
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            webviews: std::collections::HashMap::new(),
        }
    }
}

pub struct AppState(pub Mutex<BrowserState>);

#[tauri::command]
pub async fn create_tab<R: Runtime>(
    state: State<'_, AppState>,
    window: Window<R>,
) -> Result<Tab, String> {
    let tab = Tab {
        id: uuid::Uuid::new_v4().to_string(),
        url: "about:blank".to_string(),
        title: "New Tab".to_string(),
        can_go_back: false,
        can_go_forward: false,
    };

    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    
    // Create WebView
    let context = WebContext::default().unwrap();
    let webview = WebView::with_context(&context);
    webview.load_uri("about:blank");
    
    // Setup WebView events
    let tab_id = tab.id.clone();
    let window_handle = window.clone();
    webview.connect_load_changed(move |view, event| {
        match event {
            LoadEvent::Finished => {
                let can_go_back = view.can_go_back();
                let can_go_forward = view.can_go_forward();
                let uri = view.uri().unwrap_or_else(|| "about:blank".into());
                let title = view.title().unwrap_or_else(|| "New Tab".into());
                
                let _ = window_handle.emit("tab-updated", TabUpdateEvent {
                    tab_id: tab_id.clone(),
                    url: uri.to_string(),
                    title: title.to_string(),
                    can_go_back,
                    can_go_forward,
                });
            }
            _ => {}
        }
    });

    state.webviews.insert(tab.id.clone(), webview);
    state.tabs.push(tab.clone());
    state.active_tab = Some(tab.id.clone());

    Ok(tab)
}

#[tauri::command]
pub async fn close_tab(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    state.webviews.remove(&tab_id);
    state.tabs.retain(|tab| tab.id != tab_id);
    
    if state.active_tab == Some(tab_id.clone()) {
        state.active_tab = state.tabs.last().map(|tab| tab.id.clone());
    }
    
    Ok(())
}

#[tauri::command]
pub async fn navigate_to<R: Runtime>(
    window: Window<R>,
    state: State<'_, AppState>,
    url: String,
    tab_id: String,
) -> Result<(), String> {
    let state = state.0.lock().map_err(|e| e.to_string())?;
    
    if let Some(webview) = state.webviews.get(&tab_id) {
        webview.load_uri(&url);
    }
    
    Ok(())
}

#[tauri::command]
pub async fn go_back(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    let state = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(webview) = state.webviews.get(&tab_id) {
        if webview.can_go_back() {
            webview.go_back();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn go_forward(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    let state = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(webview) = state.webviews.get(&tab_id) {
        if webview.can_go_forward() {
            webview.go_forward();
        }
    }
    Ok(())
}