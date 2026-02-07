//! Pipeline module that orchestrates the caption extraction workflow.
//!
//! The pipeline combines:
//! 1. Downloading captions from YouTube via yt-dlp
//! 2. Parsing VTT content to plain text
//! 3. Optionally cleaning the text with AI (Groq)

use log::{debug, error, info, trace, warn};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::ai;
use crate::errors::CaptionError;
use crate::vtt;
use crate::yt_dlp;

/// Options for the caption extraction pipeline
#[derive(Debug, Clone)]
pub struct PipelineOptions {
    /// Whether to clean the transcript with AI
    pub use_ai_cleaning: bool,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        trace!("Creating default PipelineOptions");
        Self {
            use_ai_cleaning: false,
        }
    }
}

/// Result of the caption extraction pipeline
#[derive(Debug, Clone, Serialize)]
pub struct PipelineResult {
    /// The extracted (and optionally cleaned) transcript text
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
/// 1. Downloads captions from the given YouTube URL
/// 2. Parses the VTT file to extract plain text
/// 3. Optionally cleans the text using Groq AI
///
/// # Arguments
/// * `url` - The YouTube video URL
/// * `options` - Pipeline configuration options
///
/// # Returns
/// A `PipelineResult` containing the transcript and metadata
pub async fn run(url: &str, options: PipelineOptions) -> Result<PipelineResult, CaptionError> {
    info!("Starting pipeline for URL: {}", url);
    debug!("Pipeline options: {:?}", options);

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

    // Step 1: Download captions
    info!("Step 1/3: Downloading captions from YouTube");
    let vtt_path = yt_dlp::download_captions(url, &dir).map_err(|e| {
        error!("Failed to download captions: {}", e);
        CaptionError::Download(e)
    })?;
    info!("Captions downloaded to: {}", vtt_path.display());

    // Step 2: Read and parse VTT content
    info!("Step 2/3: Parsing VTT content");
    trace!("Reading VTT file: {}", vtt_path.display());
    let vtt_content = fs::read_to_string(&vtt_path).map_err(|e| {
        error!("Failed to read VTT file {}: {}", vtt_path.display(), e);
        CaptionError::VttRead(format!("{}: {}", vtt_path.display(), e))
    })?;
    debug!("VTT file size: {} bytes", vtt_content.len());

    trace!("Converting VTT to plain text");
    let mut transcript = vtt::vtt_to_text(&vtt_content);
    debug!(
        "Parsed transcript: {} characters, {} words",
        transcript.len(),
        transcript.split_whitespace().count()
    );

    // Clean up temp files
    debug!("Cleaning up temp files after parsing");
    cleanup_temp_dir(&dir);

    // Step 3: Optionally clean with AI
    info!(
        "Step 3/3: AI cleaning (enabled: {})",
        options.use_ai_cleaning
    );
    let ai_cleaned = if options.use_ai_cleaning && !transcript.is_empty() {
        info!("Sending transcript to AI for cleaning");
        debug!(
            "Transcript length before AI cleaning: {} chars",
            transcript.len()
        );

        match ai::clean_with_ai_from_env(&transcript).await {
            Ok(cleaned) => {
                info!("AI cleaning successful");
                debug!(
                    "Transcript length after AI cleaning: {} chars (delta: {})",
                    cleaned.len(),
                    cleaned.len() as i64 - transcript.len() as i64
                );
                transcript = cleaned;
                true
            }
            Err(e) => {
                warn!("AI cleaning failed, returning raw transcript: {}", e);
                debug!("AI error details: {:?}", e);
                false
            }
        }
    } else {
        if !options.use_ai_cleaning {
            debug!("AI cleaning disabled, skipping");
        } else if transcript.is_empty() {
            debug!("Transcript is empty, skipping AI cleaning");
        }
        false
    };

    info!(
        "Pipeline complete: {} chars, AI cleaned: {}",
        transcript.len(),
        ai_cleaned
    );

    Ok(PipelineResult {
        transcript,
        ai_cleaned,
    })
}

/// Run the pipeline without AI cleaning (synchronous convenience function)
pub fn run_sync(url: &str) -> Result<PipelineResult, CaptionError> {
    info!("Starting synchronous pipeline for URL: {}", url);

    let dir = std::env::temp_dir().join("caption_cleaner");
    debug!("Using temp directory: {}", dir.display());

    trace!("Creating temp directory if not exists");
    fs::create_dir_all(&dir).map_err(|e| {
        error!("Failed to create temp directory: {}", e);
        CaptionError::TempDirCreation(e)
    })?;

    debug!("Cleaning up previous files");
    cleanup_temp_dir(&dir);

    info!("Downloading captions");
    let vtt_path = yt_dlp::download_captions(url, &dir).map_err(|e| {
        error!("Failed to download captions: {}", e);
        CaptionError::Download(e)
    })?;
    debug!("VTT file downloaded: {}", vtt_path.display());

    info!("Reading and parsing VTT file");
    let vtt_content = fs::read_to_string(&vtt_path).map_err(|e| {
        error!("Failed to read VTT file: {}", e);
        CaptionError::VttRead(format!("{}: {}", vtt_path.display(), e))
    })?;

    let transcript = vtt::vtt_to_text(&vtt_content);
    debug!("Parsed {} characters from VTT", transcript.len());

    debug!("Cleaning up temp files");
    cleanup_temp_dir(&dir);

    info!("Synchronous pipeline complete: {} chars", transcript.len());

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
