//! Logging Module
//!
//! Structured logging with file output for diagnostics.

use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::PathBuf;

/// Initialize logging with console and file output
pub fn init() {
    let log_dir = get_log_directory();
    
    // Ensure log directory exists
    let _ = std::fs::create_dir_all(&log_dir);
    
    // Create rolling file appender (daily rotation, keep 7 days)
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "vtc-tracker.log",
    );
    
    // Create file layer
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(file_appender);
    
    // Create console layer (debug builds only)
    #[cfg(debug_assertions)]
    let console_layer = Some(
        fmt::layer()
            .with_target(true)
            .pretty()
    );
    
    #[cfg(not(debug_assertions))]
    let console_layer: Option<fmt::Layer<_>> = None;
    
    // Set up filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            #[cfg(debug_assertions)]
            { EnvFilter::new("debug,hyper=warn,reqwest=warn") }
            #[cfg(not(debug_assertions))]
            { EnvFilter::new("info,hyper=warn,reqwest=warn") }
        });
    
    // Build subscriber
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer);
    
    // Set global subscriber
    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn get_log_directory() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VTCTracker")
        .join("logs")
}
