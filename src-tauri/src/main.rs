//! VTC Tracker Desktop - Main Entry Point
//!
//! Windows desktop companion app for the VTC Tracker web platform.
//! Reads ETS2/ATS telemetry and syncs jobs automatically.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use tauri::Manager;
use tracing::info;
use std::sync::Mutex;

use vtc_tracker_lib::{
    auth::AuthManager,
    storage::SecureStorage,
    sync::ApiClient,
    telemetry::TelemetryReader,
    logging,
    commands,
    AppState,
};

fn main() {
    // Initialize logging
    logging::init();
    info!("VTC Tracker Desktop starting...");

    // Initialize application state
    let storage = SecureStorage::new();
    // TODO: Change this to your Render URL when deployed (e.g., "https://api.vtc-tracker.com")
    const DEFAULT_API_URL: &str = "http://localhost:3000";

    let api_base_url = std::env::var("VTC_API_URL")
        .unwrap_or_else(|_| DEFAULT_API_URL.to_string());
    
    let app_state = AppState {
        auth: std::sync::Mutex::new(AuthManager::new()),
        storage,
        api: ApiClient::new(&api_base_url),
        telemetry: std::sync::Mutex::new(TelemetryReader::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())

        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_stored_session,
            commands::verify_device_code,
            commands::logout,
            commands::start_telemetry,
            commands::send_heartbeat,
            commands::minimize_window,
            commands::hide_to_tray,
            commands::close_window,
        ])
        .setup(|app| {
            let tray_menu = tauri::menu::Menu::with_items(app, &[
                &tauri::menu::MenuItem::with_id(app, "show", "Show", true, None::<&str>)?,
                &tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
            ])?;

            tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            info!("Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error running VTC Tracker");
}
