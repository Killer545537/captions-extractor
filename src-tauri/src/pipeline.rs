//! Pipeline module that orchestrates the caption extraction workflow.
//!
//! The pipeline combines:
//! 1. Downloading captions from YouTube via bundled yt-dlp sidecar
//! 2. Parsing VTT content to plain text
//!
//! AI cleaning is handled at the command level, not in the pipeline.

use log::{debug, error, info, trace, warn};
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::AppHandle;

use crate::errors::CaptionError;
use crate::vtt;
use crate::yt_dlp;

/// Options for the caption extraction pipeline
#[derive(Debug, Clone, Default)]
pub struct PipelineOptions {
    /// Whether AI cleaning is requested (tracked for result metadata)
    pub use_ai_cleaning: bool,
}

/// Result of the caption extraction pipeline
#[derive(Debug, Clone, Serialize)]
pub struct PipelineResult {
    /// The extracted transcript text
    pub transcript: String,
    /// Whether AI cleaning was applied
    pub ai_cleaned: bool,
}

/// Cleans up all files in the specified directory
fn cleanup_temp_dir(dir: &Path) {
    trace!("Cleaning up temp directory: {}", dir.display());
    let mut removed_count = 0;
    let mut failed_count = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                match fs::remove_file(&path) {
                    Ok(_) => {
                        removed_count += 1;
                        trace!("Removed file: {}", path.display());
                    }
                    Err(e) => {
                        failed_count += 1;
                        warn!("Failed to remove file {}: {}", path.display(), e);
                    }
                }
            }
        }
    } else {
        debug!(
            "Could not read temp directory for cleanup: {}",
            dir.display()
        );
    }

    debug!(
        "Cleanup complete: {} files removed, {} failed",
        removed_count, failed_count
    );
}

/// Run the caption extraction pipeline.
///
/// This function:
/// 1. Downloads captions from the given YouTube URL using bundled yt-dlp
/// 2. Parses the VTT file to extract plain text
///
/// AI cleaning is NOT performed here - it's handled at the command level.
///
/// # Arguments
/// * `app` - The Tauri app handle for accessing the sidecar
/// * `url` - The YouTube video URL
/// * `_options` - Pipeline configuration options (reserved for future use)
///
/// # Returns
/// A `PipelineResult` containing the transcript and metadata
pub async fn run(
    app: &AppHandle,
    url: &str,
    _options: PipelineOptions,
) -> Result<PipelineResult, CaptionError> {
    info!("Starting pipeline for URL: {}", url);

    let dir = std::env::temp_dir().join("caption_cleaner");
    debug!("Using temp directory: {}", dir.display());

    trace!("Creating temp directory if not exists");
    fs::create_dir_all(&dir).map_err(|e| {
        error!("Failed to create temp directory {}: {}", dir.display(), e);
        CaptionError::TempDirCreation(e)
    })?;

    // Clean up any previous files before downloading
    debug!("Cleaning up previous files before download");
    cleanup_temp_dir(&dir);

    // Step 1: Download captions using sidecar
    info!("Step 1/2: Downloading captions from YouTube");
    let vtt_path = yt_dlp::download_captions_with_sidecar(app, url, &dir)
        .await
        .map_err(|e| {
            error!("Failed to download captions: {}", e);
            CaptionError::Download(e)
        })?;
    info!("Captions downloaded to: {}", vtt_path.display());

    // Step 2: Read and parse VTT content
    info!("Step 2/2: Parsing VTT content");
    trace!("Reading VTT file: {}", vtt_path.display());
    let vtt_content = fs::read_to_string(&vtt_path).map_err(|e| {
        error!("Failed to read VTT file {}: {}", vtt_path.display(), e);
        CaptionError::VttRead(format!("{}: {}", vtt_path.display(), e))
    })?;
    debug!("VTT file size: {} bytes", vtt_content.len());

    trace!("Converting VTT to plain text");
    let transcript = vtt::vtt_to_text(&vtt_content);
    debug!(
        "Parsed transcript: {} characters, {} words",
        transcript.len(),
        transcript.split_whitespace().count()
    );

    // Clean up temp files
    debug!("Cleaning up temp files after parsing");
    cleanup_temp_dir(&dir);

    info!("Pipeline complete: {} chars", transcript.len());

    Ok(PipelineResult {
        transcript,
        ai_cleaned: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_options_default() {
        let options = PipelineOptions::default();
        assert!(!options.use_ai_cleaning);
    }

    #[test]
    fn test_cleanup_temp_dir() {
        let temp_dir = std::env::temp_dir().join("caption_cleaner_pipeline_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        assert!(test_file.exists());

        cleanup_temp_dir(&temp_dir);
        assert!(!test_file.exists());

        let _ = fs::remove_dir(&temp_dir);
    }
}
