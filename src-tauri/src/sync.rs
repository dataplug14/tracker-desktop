//! API Sync Module
//!
//! Handles HTTP communication with the VTC Tracker API.

use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};

/// API client for VTC Tracker backend
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Verify a device code and get access token
    pub async fn verify_code(
        &self,
        code: &str,
        device_name: &str,
    ) -> Result<VerifyResponse, ApiError> {
        let url = format!("{}/api/auth/device/verify", self.base_url);
        
        debug!("Verifying device code at: {}", url);
        
        let response = self.client
            .post(&url)
            .json(&VerifyRequest { code, device_name })
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await
                .unwrap_or_else(|_| ErrorResponse { error: "Unknown error".into() });
            return Err(ApiError::Server(error.error));
        }
        
        let data = response.json::<VerifyResponse>().await
            .map_err(|e| ApiError::Parse(e.to_string()))?;
        
        info!("Device code verified successfully");
        Ok(data)
    }

    /// Send heartbeat to keep connection alive
    pub async fn send_heartbeat(
        &self,
        access_token: &str,
    ) -> Result<HeartbeatResponse, ApiError> {
        let url = format!("{}/api/telemetry/heartbeat", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error: ErrorResponse = response.json().await
                .unwrap_or_else(|_| ErrorResponse { error: format!("Status: {}", status) });
            return Err(ApiError::Server(error.error));
        }
        
        response.json::<HeartbeatResponse>().await
            .map_err(|e| ApiError::Parse(e.to_string()))
    }

    /// Submit a telemetry job
    pub async fn submit_job(
        &self,
        access_token: &str,
        job: &JobSubmission,
    ) -> Result<JobResponse, ApiError> {
        let url = format!("{}/api/telemetry/job", self.base_url);
        
        info!("Submitting telemetry job: {} -> {}", job.source_city, job.destination_city);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(job)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await
                .unwrap_or_else(|_| ErrorResponse { error: "Job submission failed".into() });
            return Err(ApiError::Server(error.error));
        }
        
        let data = response.json::<JobResponse>().await
            .map_err(|e| ApiError::Parse(e.to_string()))?;
        
        info!("Job submitted successfully: {}", data.job_id);
        Ok(data)
    }

    /// Disconnect (set offline)
    pub async fn disconnect(&self, access_token: &str) -> Result<(), ApiError> {
        let url = format!("{}/api/telemetry/heartbeat", self.base_url);
        
        let _ = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await;
        
        info!("Disconnected from server");
        Ok(())
    }
}

// Request/Response types

#[derive(Serialize)]
struct VerifyRequest<'a> {
    code: &'a str,
    device_name: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct VerifyResponse {
    pub access_token: String,
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatResponse {
    pub success: bool,
    pub timestamp: String,
    pub next_heartbeat_in: u32,
}

#[derive(Debug, Serialize)]
pub struct JobSubmission {
    pub game: String,
    pub cargo: String,
    pub source_city: String,
    pub destination_city: String,
    pub distance_km: u32,
    pub revenue: f64,
    pub damage_percent: f64,
    pub truck_id: Option<String>,
    pub trailer_id: Option<String>,
    pub telemetry_data: Option<serde_json::Value>,
    pub server: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JobResponse {
    pub success: bool,
    pub job_id: String,
    pub message: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

/// API errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}
