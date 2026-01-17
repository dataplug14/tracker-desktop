//! Secure Storage Module
//!
//! Handles encrypted storage using Windows DPAPI.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};

#[cfg(windows)]
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN,
};
#[cfg(windows)]
use windows::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB;

/// Secure storage using Windows DPAPI for encryption
pub struct SecureStorage {
    storage_path: PathBuf,
}

impl SecureStorage {
    /// Create new secure storage instance
    pub fn new() -> Self {
        let storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VTCTracker");
        
        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&storage_path) {
            error!("Failed to create storage directory: {}", e);
        }
        
        debug!("Secure storage initialized at: {:?}", storage_path);
        
        Self { storage_path }
    }

    /// Save data securely using DPAPI
    pub fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<(), StorageError> {
        let json = serde_json::to_string(data)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        let encrypted = self.encrypt(json.as_bytes())?;
        
        let file_path = self.storage_path.join(format!("{}.dat", key));
        std::fs::write(&file_path, encrypted)
            .map_err(|e| StorageError::Io(e.to_string()))?;
        
        info!("Saved encrypted data for key: {}", key);
        Ok(())
    }

    /// Load data securely using DPAPI
    pub fn load<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T, StorageError> {
        let file_path = self.storage_path.join(format!("{}.dat", key));
        
        let encrypted = std::fs::read(&file_path)
            .map_err(|e| StorageError::Io(e.to_string()))?;
        
        let decrypted = self.decrypt(&encrypted)?;
        
        let json = String::from_utf8(decrypted)
            .map_err(|e| StorageError::Decryption(e.to_string()))?;
        
        serde_json::from_str(&json)
            .map_err(|e| StorageError::Serialization(e.to_string()))
    }

    /// Delete stored data
    pub fn delete(&self, key: &str) -> Result<(), StorageError> {
        let file_path = self.storage_path.join(format!("{}.dat", key));
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| StorageError::Io(e.to_string()))?;
            info!("Deleted stored data for key: {}", key);
        }
        
        Ok(())
    }

    /// Check if key exists
    pub fn exists(&self, key: &str) -> bool {
        let file_path = self.storage_path.join(format!("{}.dat", key));
        file_path.exists()
    }

    #[cfg(windows)]
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        use std::ptr::null_mut;
        
        let input = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };
        
        unsafe {
            let result = CryptProtectData(
                &input,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            );
            
            if result.is_err() {
                return Err(StorageError::Encryption("DPAPI encryption failed".into()));
            }
            
            let encrypted = std::slice::from_raw_parts(
                output.pbData,
                output.cbData as usize,
            ).to_vec();
            
            // Free the memory allocated by CryptProtectData
            windows::Win32::Foundation::LocalFree(
                windows::Win32::Foundation::HLOCAL(output.pbData as *mut std::ffi::c_void)
            );
            
            Ok(encrypted)
        }
    }

    #[cfg(windows)]
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        use std::ptr::null_mut;
        
        let input = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };
        
        unsafe {
            let result = CryptUnprotectData(
                &input,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            );
            
            if result.is_err() {
                return Err(StorageError::Decryption("DPAPI decryption failed".into()));
            }
            
            let decrypted = std::slice::from_raw_parts(
                output.pbData,
                output.cbData as usize,
            ).to_vec();
            
            // Free the memory allocated by CryptUnprotectData
            windows::Win32::Foundation::LocalFree(
                windows::Win32::Foundation::HLOCAL(output.pbData as *mut std::ffi::c_void)
            );
            
            Ok(decrypted)
        }
    }

    #[cfg(not(windows))]
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        // Fallback for non-Windows (development only)
        Ok(data.to_vec())
    }

    #[cfg(not(windows))]
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        // Fallback for non-Windows (development only)
        Ok(data.to_vec())
    }
}

impl Default for SecureStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
}
