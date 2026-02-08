//! Caption Extractor - A Tauri application for extracting and cleaning YouTube captions.

use log::{debug, error, info, trace, warn};
use std::path::PathBuf;
use tauri::AppHandle;

mod ai;
mod errors;
mod pipeline;
mod settings;
mod vtt;
mod yt_dlp;

pub use errors::CaptionError;

use pipeline::{PipelineOptions, PipelineResult};

/// Extract captions from a YouTube URL without AI cleaning.
///
/// This is the basic extraction command that downloads captions and converts them to plain text.
#[tauri::command]
async fn extract_captions(app: AppHandle, url: String) -> Result<String, CaptionError> {
    info!("extract_captions called with URL: {}", url);
    trace!("Starting caption extraction without AI cleaning");

    match pipeline::run(&app, &url, PipelineOptions::default()).await {
        Ok(result) => {
            info!(
                "Successfully extracted captions: {} characters",
                result.transcript.len()
            );
            debug!(
                "Transcript preview: {}...",
                &result.transcript.chars().take(100).collect::<String>()
            );
            Ok(result.transcript)
        }
        Err(e) => {
            error!("Failed to extract captions: {}", e);
            Err(e)
        }
    }
}

/// Extract captions from a YouTube URL with optional AI cleaning.
///
/// When `use_ai` is true, the transcript will be cleaned using the Groq API
/// to fix spelling, grammar, and punctuation while preserving the original meaning.
///
/// Uses the stored API key or falls back to the `GROQ_API_KEY` environment variable.
#[tauri::command]
async fn extract_captions_with_options(
    app: AppHandle,
    url: String,
    use_ai: bool,
) -> Result<PipelineResult, CaptionError> {
    info!(
        "extract_captions_with_options called with URL: {}, use_ai: {}",
        url, use_ai
    );

    let options = PipelineOptions {
        use_ai_cleaning: use_ai,
    };

    trace!("Pipeline options: {:?}", options);

    // Run the pipeline
    let mut result = pipeline::run(&app, &url, options.clone()).await?;

    // If AI cleaning is requested, apply it using the app's API key
    if use_ai && !result.ai_cleaned {
        info!("Applying AI cleaning to extracted transcript");
        match ai::clean_with_ai_from_app(&app, &result.transcript).await {
            Ok(cleaned) => {
                result.transcript = cleaned;
                result.ai_cleaned = true;
                info!("AI cleaning applied successfully");
            }
            Err(e) => {
                warn!("AI cleaning failed, returning uncleaned transcript: {}", e);
                // Don't fail the whole operation, just skip AI cleaning
            }
        }
    }

    info!(
        "Successfully extracted captions: {} characters, AI cleaned: {}",
        result.transcript.len(),
        result.ai_cleaned
    );
    debug!(
        "Transcript preview: {}...",
        &result.transcript.chars().take(100).collect::<String>()
    );

    Ok(result)
}

/// Check if the Groq API key is configured.
///
/// Returns true if an API key is stored in settings or set in the environment.
#[tauri::command]
fn is_ai_available(app: AppHandle) -> bool {
    trace!("is_ai_available called");
    let available = settings::is_api_key_configured(&app);
    debug!("AI availability check result: {}", available);

    if !available {
        info!("No API key configured - AI cleaning disabled");
    } else {
        trace!("API key is configured");
    }

    available
}

/// Clean existing transcript text using AI.
///
/// This command can be used to clean a transcript that was previously extracted
/// without AI cleaning.
#[tauri::command]
async fn clean_transcript_with_ai(app: AppHandle, text: String) -> Result<String, CaptionError> {
    info!(
        "clean_transcript_with_ai called with {} characters",
        text.len()
    );
    trace!(
        "Input text preview: {}...",
        &text.chars().take(100).collect::<String>()
    );

    match ai::clean_with_ai_from_app(&app, &text).await {
        Ok(cleaned) => {
            info!(
                "Successfully cleaned transcript: {} -> {} characters",
                text.len(),
                cleaned.len()
            );
            debug!(
                "Cleaned text preview: {}...",
                &cleaned.chars().take(100).collect::<String>()
            );
            Ok(cleaned)
        }
        Err(e) => {
            error!("Failed to clean transcript with AI: {}", e);
            Err(e)
        }
    }
}

/// Save the Groq API key to persistent storage.
#[tauri::command]
fn save_api_key(app: AppHandle, api_key: String) -> Result<(), CaptionError> {
    info!("save_api_key called");

    if api_key.trim().is_empty() {
        warn!("Attempted to save empty API key");
        return Err(CaptionError::SettingsError(
            "API key cannot be empty".to_string(),
        ));
    }

    settings::save_api_key(&app, api_key.trim())?;
    info!("API key saved successfully");
    Ok(())
}

/// Get the current API key (masked for security).
///
/// Returns the first 8 and last 4 characters with the middle masked.
#[tauri::command]
fn get_api_key_masked(app: AppHandle) -> Option<String> {
    trace!("get_api_key_masked called");

    settings::get_effective_api_key(&app).map(|key| {
        if key.len() <= 12 {
            // Very short key, just show asterisks
            "*".repeat(key.len())
        } else {
            // Show first 8 and last 4 characters
            let prefix = &key[..8];
            let suffix = &key[key.len() - 4..];
            let masked_len = key.len() - 12;
            format!("{}{}...{}", prefix, "*".repeat(masked_len.min(8)), suffix)
        }
    })
}

/// Remove the stored API key.
#[tauri::command]
fn remove_api_key(app: AppHandle) -> Result<(), CaptionError> {
    info!("remove_api_key called");
    settings::remove_api_key(&app)?;
    info!("API key removed successfully");
    Ok(())
}

/// Check if the API key is from stored settings (vs environment).
#[tauri::command]
fn is_api_key_stored(app: AppHandle) -> bool {
    settings::get_api_key(&app).is_some()
}

/// Load environment variables from .env file.
/// Searches in multiple locations to find the .env file.
fn load_dotenv() {
    trace!("Attempting to load .env file");

    // Try to load from current directory first
    if dotenvy::dotenv().is_ok() {
        info!("Loaded .env file from current directory");
        return;
    }

    // Try to load from the executable's directory (for packaged apps)
    if let Ok(exe_path) = std::env::current_exe() {
        debug!("Executable path: {}", exe_path.display());

        if let Some(exe_dir) = exe_path.parent() {
            let env_path = exe_dir.join(".env");
            trace!("Checking for .env at: {}", env_path.display());

            if env_path.exists() {
                if dotenvy::from_path(&env_path).is_ok() {
                    info!(
                        "Loaded .env file from executable directory: {}",
                        env_path.display()
                    );
                    return;
                }
            }

            // Also try parent directory (for macOS .app bundles)
            if let Some(parent) = exe_dir.parent() {
                let env_path = parent.join(".env");
                trace!("Checking for .env at: {}", env_path.display());

                if env_path.exists() {
                    if dotenvy::from_path(&env_path).is_ok() {
                        info!(
                            "Loaded .env file from parent directory: {}",
                            env_path.display()
                        );
                        return;
                    }
                }

                // Try two levels up (for macOS .app/Contents/MacOS structure)
                if let Some(grandparent) = parent.parent() {
                    if let Some(great_grandparent) = grandparent.parent() {
                        let env_path = great_grandparent.join(".env");
                        trace!("Checking for .env at: {}", env_path.display());

                        if env_path.exists() {
                            if dotenvy::from_path(&env_path).is_ok() {
                                info!(
                                    "Loaded .env file from app bundle root: {}",
                                    env_path.display()
                                );
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    // Try common development paths
    let dev_paths: Vec<PathBuf> = vec![
        PathBuf::from("../.env"),    // From src-tauri directory
        PathBuf::from("../../.env"), // Two levels up
    ];

    for path in dev_paths {
        trace!("Checking for .env at development path: {}", path.display());
        if path.exists() {
            if dotenvy::from_path(&path).is_ok() {
                info!("Loaded .env file from development path: {}", path.display());
                return;
            }
        }
    }

    debug!("No .env file found in any searched location");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Caption Extractor starting up");
    debug!("Initializing application");

    // Load .env file before starting the application
    load_dotenv();

    // Log environment API key status (stored key status logged after app starts)
    if std::env::var("GROQ_API_KEY").is_ok() {
        info!("GROQ_API_KEY found in environment");
    }

    info!("Building Tauri application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            extract_captions,
            extract_captions_with_options,
            is_ai_available,
            clean_transcript_with_ai,
            save_api_key,
            get_api_key_masked,
            remove_api_key,
            is_api_key_stored,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ai_available_without_app() {
        // This test just verifies the module compiles correctly
        // Full integration tests would require a Tauri app context
    }
}
