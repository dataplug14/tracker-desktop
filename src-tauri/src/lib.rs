//! VTC Tracker Desktop Library
//!
//! Core modules for the desktop companion app.

pub mod auth;
pub mod storage;
pub mod sync;
pub mod telemetry;
pub mod logging;
pub mod commands;

use std::sync::Mutex;
use auth::AuthManager;
use storage::SecureStorage;
use sync::ApiClient;
use telemetry::TelemetryReader;

/// Application state shared across commands
pub struct AppState {
    pub auth: Mutex<AuthManager>,
    pub storage: SecureStorage,
    pub api: ApiClient,
    pub telemetry: Mutex<TelemetryReader>,
}
