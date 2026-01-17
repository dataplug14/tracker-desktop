//! Authentication Module
//!
//! Handles device token management and session state.

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Session data stored securely on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_at
    }
}

/// Manages authentication state
pub struct AuthManager {
    session: Option<Session>,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Self {
        Self { session: None }
    }

    /// Set the current session
    pub fn set_session(&mut self, session: Session) {
        info!("Session set for user: {}", session.user_id);
        self.session = Some(session);
    }

    /// Get the current session if valid
    pub fn get_session(&self) -> Option<&Session> {
        match &self.session {
            Some(session) if !session.is_expired() => Some(session),
            Some(_) => {
                warn!("Session is expired");
                None
            }
            None => None,
        }
    }

    /// Get the access token if authenticated
    pub fn get_access_token(&self) -> Option<&str> {
        self.get_session().map(|s| s.access_token.as_str())
    }

    /// Check if currently authenticated
    pub fn is_authenticated(&self) -> bool {
        self.get_session().is_some()
    }

    /// Clear the current session
    pub fn clear_session(&mut self) {
        info!("Session cleared");
        self.session = None;
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
