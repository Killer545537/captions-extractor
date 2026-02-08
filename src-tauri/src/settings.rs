//! Settings management for persistent configuration storage.
//!
//! This module handles storing and retrieving user settings like API keys
//! using Tauri's plugin-store for secure, persistent storage.

use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::errors::CaptionError;

/// Store file name for settings
const STORE_FILE: &str = "settings.json";

/// Key for storing the Groq API key
const GROQ_API_KEY_STORE_KEY: &str = "groq_api_key";

/// In-memory cache for the API key to avoid repeated store reads
static API_KEY_CACHE: RwLock<Option<String>> = RwLock::new(None);

/// Application settings structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Groq API key for AI cleaning features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groq_api_key: Option<String>,
}

/// Saves the Groq API key to persistent storage.
///
/// The key is stored using Tauri's plugin-store which persists data
/// in the app's data directory.
pub fn save_api_key(app: &AppHandle, api_key: &str) -> Result<(), CaptionError> {
    info!("Saving API key to store");
    trace!("API key length: {} characters", api_key.len());

    let store = app.store(STORE_FILE).map_err(|e| {
        error!("Failed to open settings store: {}", e);
        CaptionError::SettingsError(format!("Failed to open settings store: {}", e))
    })?;

    // Save to store
    store.set(GROQ_API_KEY_STORE_KEY, api_key.to_string());

    store.save().map_err(|e| {
        error!("Failed to save settings store: {}", e);
        CaptionError::SettingsError(format!("Failed to save settings: {}", e))
    })?;

    // Update cache
    if let Ok(mut cache) = API_KEY_CACHE.write() {
        *cache = Some(api_key.to_string());
        debug!("Updated API key cache");
    }

    info!("API key saved successfully");
    Ok(())
}

/// Retrieves the Groq API key from persistent storage.
///
/// Returns None if no API key has been saved.
pub fn get_api_key(app: &AppHandle) -> Option<String> {
    trace!("Retrieving API key");

    // Check cache first
    if let Ok(cache) = API_KEY_CACHE.read() {
        if let Some(ref key) = *cache {
            trace!("Returning API key from cache");
            return Some(key.clone());
        }
    }

    // Load from store
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(e) => {
            debug!("Failed to open settings store: {}", e);
            return None;
        }
    };

    let api_key: Option<String> = store
        .get(GROQ_API_KEY_STORE_KEY)
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    // Update cache if we found a key
    if let Some(ref key) = api_key {
        if let Ok(mut cache) = API_KEY_CACHE.write() {
            *cache = Some(key.clone());
            debug!("Cached API key from store");
        }
    }

    api_key
}

/// Removes the Groq API key from persistent storage.
pub fn remove_api_key(app: &AppHandle) -> Result<(), CaptionError> {
    info!("Removing API key from store");

    let store = app.store(STORE_FILE).map_err(|e| {
        error!("Failed to open settings store: {}", e);
        CaptionError::SettingsError(format!("Failed to open settings store: {}", e))
    })?;

    store.delete(GROQ_API_KEY_STORE_KEY);

    store.save().map_err(|e| {
        error!("Failed to save settings store: {}", e);
        CaptionError::SettingsError(format!("Failed to save settings: {}", e))
    })?;

    // Clear cache
    if let Ok(mut cache) = API_KEY_CACHE.write() {
        *cache = None;
        debug!("Cleared API key cache");
    }

    info!("API key removed successfully");
    Ok(())
}

/// Checks if an API key is configured (either in store or environment).
///
/// Priority:
/// 1. Stored API key (from settings)
/// 2. Environment variable (GROQ_API_KEY)
pub fn is_api_key_configured(app: &AppHandle) -> bool {
    // Check stored key first
    if get_api_key(app).is_some() {
        trace!("API key found in store");
        return true;
    }

    // Fall back to environment variable
    if std::env::var("GROQ_API_KEY").is_ok() {
        trace!("API key found in environment");
        return true;
    }

    trace!("No API key configured");
    false
}

/// Gets the effective API key (stored or from environment).
///
/// Priority:
/// 1. Stored API key (from settings)
/// 2. Environment variable (GROQ_API_KEY)
pub fn get_effective_api_key(app: &AppHandle) -> Option<String> {
    // Check stored key first
    if let Some(key) = get_api_key(app) {
        debug!("Using API key from store");
        return Some(key);
    }

    // Fall back to environment variable
    if let Ok(key) = std::env::var("GROQ_API_KEY") {
        debug!("Using API key from environment");
        return Some(key);
    }

    debug!("No API key available");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(settings.groq_api_key.is_none());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings {
            groq_api_key: Some("test-key".to_string()),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("groq_api_key"));
        assert!(json.contains("test-key"));
    }

    #[test]
    fn test_settings_deserialization() {
        let json = r#"{"groq_api_key": "test-key"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.groq_api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_settings_empty_deserialization() {
        let json = r#"{}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(settings.groq_api_key.is_none());
    }
}
