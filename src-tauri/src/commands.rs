//! Tauri Commands Module
//!
//! IPC commands exposed to the frontend.

use tauri::{command, State, Manager, AppHandle, WebviewWindow, Emitter};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};

use crate::AppState;
use crate::auth::Session;

// Response types for frontend

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub access_token: String,
    pub user_id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub success: bool,
    pub access_token: Option<String>,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResult {
    pub success: bool,
}

// Commands

/// Get stored session from secure storage
#[command]
pub fn get_stored_session(state: State<'_, AppState>) -> Option<SessionResponse> {
    debug!("Getting stored session");
    
    // Try to load from secure storage
    match state.storage.load::<Session>("session") {
        Ok(session) => {
            if session.is_expired() {
                info!("Stored session is expired");
                let _ = state.storage.delete("session");
                return None;
            }
            
            // Update auth manager
            if let Ok(mut auth) = state.auth.lock() {
                auth.set_session(session.clone());
            }
            
            Some(SessionResponse {
                access_token: session.access_token,
                user_id: session.user_id,
                display_name: session.display_name,
            })
        }
        Err(_) => {
            debug!("No stored session found");
            None
        }
    }
}

/// Verify device code and authenticate
#[command]
pub async fn verify_device_code(
    code: String,
    state: State<'_, AppState>,
) -> Result<VerifyResult, String> {
    info!("Verifying device code: {}", &code[..2]); // Only log first 2 chars
    
    // Get device name
    let device_name = whoami::fallible::hostname()
        .unwrap_or_else(|_| "VTC Desktop".to_string());
    
    match state.api.verify_code(&code, &device_name).await {
        Ok(response) => {
            // Parse expiration
            let expires_at = chrono::DateTime::parse_from_rfc3339(&response.expires_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| {
                    chrono::Utc::now() + chrono::Duration::days(30)
                });
            
            // Create session
            let session = Session {
                access_token: response.access_token.clone(),
                user_id: response.user_id.clone(),
                display_name: response.display_name.clone(),
                avatar_url: response.avatar_url,
                expires_at,
            };
            
            // Update auth manager
            if let Ok(mut auth) = state.auth.lock() {
                auth.set_session(session.clone());
            }
            
            // Save to secure storage
            if let Err(e) = state.storage.save("session", &session) {
                error!("Failed to save session: {}", e);
            }
            
            Ok(VerifyResult {
                success: true,
                access_token: Some(response.access_token),
                user_id: Some(response.user_id),
                display_name: Some(response.display_name),
                error: None,
            })
        }
        Err(e) => {
            error!("Code verification failed: {}", e);
            Ok(VerifyResult {
                success: false,
                access_token: None,
                user_id: None,
                display_name: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Logout and clear session
#[command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    info!("Logging out");
    
    // Get token before clearing
    let token = state.auth.lock()
        .ok()
        .and_then(|auth| auth.get_access_token().map(|s| s.to_string()));
    
    // Notify server
    if let Some(token) = token {
        let _ = state.api.disconnect(&token).await;
    }
    
    // Clear auth manager
    if let Ok(mut auth) = state.auth.lock() {
        auth.clear_session();
    }
    
    // Delete stored session
    let _ = state.storage.delete("session");
    
    Ok(())
}

/// Start telemetry reader
#[command]
pub fn start_telemetry(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    debug!("Starting telemetry");
    
    // Check if already running?
    // For simplicity, we just spawn. A better way uses atomic bool or similar.
    // But since this is usually called once on mount...
    
    let app_handle = app.clone();
    let state_handle = state.inner().clone(); // AppState likely needs to be Clone or wrapped in Arc? 
    // AppState fields are Mutex/Arc safe. But AppState struct itself is not Clone/Arc'd by default in Tauri management?
    // Actually `State` wraps it. `state.inner()` gives reference.
    // We need to clone the Arcs inside AppState. "inner().clone()" works if AppState implements Clone.
    // Let's check lib.rs for AppState definition. It has Mutex field. Mutex is not Clone.
    // We need Arc<Mutex<...>>.
    // In lib.rs: pub auth: Mutex<AuthManager>. NOT Arc.
    // This is a problem for spawning tasks. The State stays alive, but we can't move reference into 'static task.
    // We can use `app_handle.state::<AppState>()` inside the task? Yes.
    
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            let state = app_handle.state::<AppState>();
            let mut event_to_emit: Option<crate::telemetry::TelemetryEvent> = None;
            let mut telemetry_data: Option<crate::telemetry::TelemetryState> = None;
            
            // 1. Update Telemetry
            if let Ok(mut telemetry) = state.telemetry.lock() {
                 if let Some(event) = telemetry.update() {
                     event_to_emit = Some(event);
                 }
                 telemetry_data = Some(telemetry.get_state().clone());
            }
            
            // 2. Emit to Frontend
            if let Some(data) = telemetry_data {
                // Emit raw state, or specific event? 
                // Dashboard expects current state.
                let _ = app_handle.emit("telemetry_update", &data);
            }
            
            // 3. Handle Events (Sync)
            if let Some(event) = event_to_emit {
                match event {
                    crate::telemetry::TelemetryEvent::Connected(game) => {
                         info!("Game connected: {}", game);
                    }
                    crate::telemetry::TelemetryEvent::Disconnected => {
                        info!("Game disconnected");
                    }
                    crate::telemetry::TelemetryEvent::JobCompleted(job) => {
                        info!("Job completed: {} -> {}", job.source_city, job.destination_city);
                        
                        // Submit to API
                        // We need token
                        let token = state.auth.lock()
                            .ok()
                            .and_then(|auth| auth.get_access_token().map(|s| s.to_string()));
                            
                        if let Some(token) = token {
                            // Construct submission
                            // We need to map ActiveJob to JobSubmission
                             let submission = crate::sync::JobSubmission {
                                 game: "ets2".to_string(), // TODO: Get from telemetry state
                                 cargo: job.cargo.clone(),
                                 source_city: job.source_city.clone(),
                                 destination_city: job.destination_city.clone(),
                                 distance_km: job.distance_km,
                                 revenue: job.revenue as f64,
                                 damage_percent: 0.0, // TODO: Read damage
                                 truck_id: None,
                                 trailer_id: None,
                                 telemetry_data: None,
                                 server: None,
                             };
                             
                             // Spawn sync to avoid blocking loop?
                             // submit_job is async, we are in async task.
                             if let Err(e) = state.api.submit_job(&token, &submission).await {
                                 error!("Failed to submit job: {}", e);
                             }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
    
    Ok(())
}

/// Send heartbeat to server
#[command]
pub async fn send_heartbeat(state: State<'_, AppState>) -> Result<HeartbeatResult, String> {
    let token = state.auth.lock()
        .ok()
        .and_then(|auth| auth.get_access_token().map(|s| s.to_string()));
    
    let Some(token) = token else {
        return Ok(HeartbeatResult { success: false });
    };
    
    match state.api.send_heartbeat(&token).await {
        Ok(response) => Ok(HeartbeatResult { success: response.success }),
        Err(e) => {
            debug!("Heartbeat failed: {}", e);
            Ok(HeartbeatResult { success: false })
        }
    }
}

/// Minimize window
#[command]
pub fn minimize_window(window: WebviewWindow) {
    let _ = window.minimize();
}

/// Hide to system tray
#[command]
pub fn hide_to_tray(window: WebviewWindow) {
    let _ = window.hide();
}

/// Close window
#[command]
pub fn close_window(app: AppHandle) {
    app.exit(0);
}
