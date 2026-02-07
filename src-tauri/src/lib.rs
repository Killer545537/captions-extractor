//! Caption Extractor - A Tauri application for extracting and cleaning YouTube captions.

use log::{debug, error, info, trace, warn};
use std::path::PathBuf;

mod ai;
mod errors;
mod pipeline;
mod vtt;
mod yt_dlp;

pub use errors::CaptionError;

use pipeline::{PipelineOptions, PipelineResult};

/// Extract captions from a YouTube URL without AI cleaning.
///
/// This is the basic extraction command that downloads captions and converts them to plain text.
#[tauri::command]
async fn extract_captions(url: String) -> Result<String, CaptionError> {
    info!("extract_captions called with URL: {}", url);
    trace!("Starting caption extraction without AI cleaning");

    match pipeline::run(&url, PipelineOptions::default()).await {
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
/// Requires the `GROQ_API_KEY` environment variable to be set when using AI cleaning.
#[tauri::command]
async fn extract_captions_with_options(
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

    match pipeline::run(&url, options).await {
        Ok(result) => {
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
        Err(e) => {
            error!("Failed to extract captions with options: {}", e);
            Err(e)
        }
    }
}

/// Check if the Groq API key is configured.
///
/// Returns true if the `GROQ_API_KEY` environment variable is set.
#[tauri::command]
fn is_ai_available() -> bool {
    trace!("is_ai_available called");
    let available = std::env::var("GROQ_API_KEY").is_ok();
    debug!("AI availability check result: {}", available);

    if !available {
        info!("GROQ_API_KEY not found in environment - AI cleaning disabled");
    } else {
        trace!("GROQ_API_KEY found in environment");
    }

    available
}

/// Clean existing transcript text using AI.
///
/// This command can be used to clean a transcript that was previously extracted
/// without AI cleaning.
#[tauri::command]
async fn clean_transcript_with_ai(text: String) -> Result<String, CaptionError> {
    info!(
        "clean_transcript_with_ai called with {} characters",
        text.len()
    );
    trace!(
        "Input text preview: {}...",
        &text.chars().take(100).collect::<String>()
    );

    match ai::clean_with_ai_from_env(&text).await {
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

    warn!("No .env file found in any searched location");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Caption Extractor starting up");
    debug!("Initializing application");

    // Load .env file before starting the application
    load_dotenv();

    // Log AI availability status
    if std::env::var("GROQ_API_KEY").is_ok() {
        info!("GROQ_API_KEY is configured - AI cleaning available");
    } else {
        warn!("GROQ_API_KEY not set - AI cleaning will be disabled");
    }

    info!("Building Tauri application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            extract_captions,
            extract_captions_with_options,
            is_ai_available,
            clean_transcript_with_ai,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ai_available_without_key() {
        // This test assumes GROQ_API_KEY is not set in the test environment
        // If it is set, this test should still pass as it just checks the function works
        let _ = is_ai_available();
    }
}
