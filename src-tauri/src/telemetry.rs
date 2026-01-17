//! Telemetry Module
//!
//! Reads ETS2/ATS telemetry from shared memory using Windows API.
//! This manual implementation avoids external crate dependency issues (bindgen/libclang).

use serde::{Deserialize, Serialize};
use tracing::{info, debug, error};

#[cfg(windows)]
use windows::Win32::Foundation::{HANDLE, CloseHandle};
#[cfg(windows)]
use windows::Win32::System::Memory::{
    OpenFileMappingA, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_ALL_ACCESS,
};

/// Game type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Ets2,
    Ats,
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::Ets2 => write!(f, "ets2"),
            Game::Ats => write!(f, "ats"),
        }
    }
}

/// Current telemetry state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryState {
    pub connected: bool,
    pub game: Option<Game>,
    pub speed: f32,
    pub current_city: Option<String>,
    pub active_job: Option<ActiveJob>,
}

impl Default for TelemetryState {
    fn default() -> Self {
        Self {
            connected: false,
            game: None,
            speed: 0.0,
            current_city: None,
            active_job: None,
        }
    }
}

/// Active job information from telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveJob {
    pub cargo: String,
    pub source_city: String,
    pub destination_city: String,
    pub distance_km: u32,
    pub distance_remaining: u32,
    pub revenue: u64,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

// SCS Telemetry Memory Map Layout (Simplified/Partial)
// Based on typical scs-sdk-plugin layout.
// WARNING: Offsets may vary by version. This is a best-effort mapping.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScsHeader {
    paused: u32,      // 0x00
    paused_time: u32, // 0x04
    game_timestamp: u32, // 0x08
    active: u32,      // 0x0C - SDK Active?
}

// Full struct would be large. We just read header to check connection.
// To read speed/job, we'd need exact offsets.
// For now, we implement connection detection which is the critical first step.

pub struct TelemetryReader {
    state: TelemetryState,
    #[cfg(windows)]
    map_handle: HANDLE,
    #[cfg(windows)]
    map_view: *const std::ffi::c_void,
    job_started: bool,
}

impl TelemetryReader {
    pub fn new() -> Self {
        Self {
            state: TelemetryState::default(),
            #[cfg(windows)]
            map_handle: HANDLE::default(),
            #[cfg(windows)]
            map_view: std::ptr::null(),
            job_started: false,
        }
    }

    pub fn get_state(&self) -> &TelemetryState {
        &self.state
    }

    pub fn connect(&mut self) -> bool {
        #[cfg(windows)]
        {
            if !self.map_handle.is_invalid() && !self.map_view.is_null() {
                return true;
            }

            unsafe {
                let name = std::ffi::CString::new("Local\\SCSTelemetry").unwrap();
                let handle = OpenFileMappingA(
                    FILE_MAP_READ.0, // Read access
                    false,
                    windows::core::PCSTR(name.as_ptr() as *const u8),
                );

                if let Ok(handle) = handle {
                    if handle.is_invalid() {
                         // Failed to open
                         return false;
                    }

                    let view = MapViewOfFile(
                        handle,
                        FILE_MAP_READ,
                        0,
                        0,
                        0, // Map entire file
                    );

                    if view.Value.is_null() {
                        CloseHandle(handle);
                        return false;
                    }

                    info!("Connected to SCS Telemetry Shared Memory");
                    self.map_handle = handle;
                    self.map_view = view.Value;
                    self.state.connected = true;
                    true
                } else {
                    self.state.connected = false;
                    false
                }
            }
        }
        #[cfg(not(windows))]
        {
            false
        }
    }
    
    // Safety: We implement Drop to clean up handles
    #[cfg(windows)]
    fn cleanup(&mut self) {
        unsafe {
            if !self.map_view.is_null() {
                let _ = UnmapViewOfFile(windows::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS { Value: self.map_view as *mut _ });
                self.map_view = std::ptr::null();
            }
            if !self.map_handle.is_invalid() {
                let _ = CloseHandle(self.map_handle);
                self.map_handle = HANDLE::default();
            }
        }
    }

    pub fn update(&mut self) -> Option<TelemetryEvent> {
        #[cfg(windows)]
        {
            if self.state.connected {
                if self.map_view.is_null() {
                    self.state.connected = false;
                    return Some(TelemetryEvent::Disconnected);
                }

                unsafe {
                    // Check if we can read the header
                    // We interpret the first few bytes as header
                    let header_ptr = self.map_view as *const ScsHeader;
                    let _header = *header_ptr; // Copy header
                    
                    // Simple check: if paused is 0 or 1, and active is set?
                    // We assume connection is valid if we have the view.
                    // But if game closes, the handle might remain valid but stale?
                    // Actually OpenFileMapping fails if game is not running (usually).
                    
                    // TODO: Implement full data reading once exact struct layout is confirmed.
                    // For now, connection status is what we guarantee.
                    
                    // If we wanted to read speed, we need offset.
                    // Let's assume standard offset for v1.12 plugin:
                    // speed is usually at some offset.
                    // Without exact offset, reading implies risk of garbage data.
                    // Better to show 0 speed than random numbers.
                }
                
                return None;
            } else {
                // Try to connect
                if self.connect() {
                     // Default to ETS2 if we connect
                    self.state.game = Some(Game::Ets2);
                    return Some(TelemetryEvent::Connected(Game::Ets2));
                }
            }
        }
        None
    }
}

// Safety: TelemetryReader manages a thread-safe file mapping handle and view.
// Access is synchronized via the Mutex wrapper in AppState.
unsafe impl Send for TelemetryReader {}
unsafe impl Sync for TelemetryReader {}

#[cfg(windows)]
impl Drop for TelemetryReader {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl Default for TelemetryReader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum TelemetryEvent {
    Connected(Game),
    Disconnected,
    JobStarted,
    JobCompleted(ActiveJob),
}
