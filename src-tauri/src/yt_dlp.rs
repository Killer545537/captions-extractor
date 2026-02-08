//! yt-dlp integration module for downloading YouTube captions.
//!
//! This module provides functionality to download captions from YouTube videos
//! using the bundled yt-dlp sidecar binary.

use log::{debug, error, info, trace, warn};
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YtDlpError {
    #[error("yt-dlp sidecar not found")]
    NotInstalled,

    #[error("Failed to execute yt-dlp: {0}")]
    ExecutionFailed(String),

    #[error("yt-dlp failed: {0}")]
    CommandFailed(String),

    #[error("No subtitles found for this video")]
    NoSubtitlesFound,

    #[error("Failed to read output directory: {0}")]
    OutputDirReadFailed(String),
}

/// Download captions for a video URL using the bundled yt-dlp sidecar.
///
/// This function uses Tauri's sidecar functionality to run the bundled yt-dlp binary.
pub async fn download_captions_with_sidecar(
    app: &tauri::AppHandle,
    url: &str,
    out_dir: &Path,
) -> Result<PathBuf, YtDlpError> {
    info!("Starting caption download for URL: {}", url);
    debug!("Output directory: {}", out_dir.display());

    // Clean up any existing VTT files in the output directory to avoid confusion
    debug!("Cleaning up existing VTT files in output directory");
    cleanup_vtt_files(out_dir);

    let output_template = format!("{}/%(id)s.%(ext)s", out_dir.display());
    trace!("Output template: {}", output_template);

    let args = [
        "--write-subs",
        "--write-auto-subs",
        "--sub-langs",
        "en.*,en",
        "--skip-download",
        "--no-playlist",
        "-o",
        &output_template,
        url,
    ];

    info!("Executing yt-dlp sidecar command");
    debug!("yt-dlp arguments: {:?}", args);

    // Use the sidecar command
    let shell = app.shell();
    let output = shell
        .sidecar("yt-dlp")
        .map_err(|e| {
            error!("Failed to create yt-dlp sidecar command: {}", e);
            YtDlpError::NotInstalled
        })?
        .args(args)
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute yt-dlp sidecar: {}", e);
            YtDlpError::ExecutionFailed(e.to_string())
        })?;

    trace!("yt-dlp exit status: {:?}", output.status);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        warn!("yt-dlp command failed with status: {:?}", output.status);
        debug!("yt-dlp stderr:\n{}", stderr);
        debug!("yt-dlp stdout:\n{}", stdout);

        // Try to provide a more helpful error message
        let error_msg = if stderr.contains("Video unavailable") {
            error!("Video is unavailable or private");
            "Video is unavailable or private".to_string()
        } else if stderr.contains("is not a valid URL") {
            error!("Invalid URL provided: {}", url);
            "Invalid URL provided".to_string()
        } else if stderr.contains("Private video") {
            error!("Video is private");
            "Video is private".to_string()
        } else if stderr.contains("Sign in to confirm your age") {
            error!("Video requires age verification");
            "Video requires age verification".to_string()
        } else if stderr.contains("No video formats found") {
            error!("No video formats found - video may be unavailable in your region");
            "No video formats found".to_string()
        } else if !stderr.is_empty() {
            let first_line = stderr.lines().next().unwrap_or("Unknown error");
            error!("yt-dlp error: {}", first_line);
            first_line.to_string()
        } else if !stdout.is_empty() {
            let first_line = stdout.lines().next().unwrap_or("Unknown error");
            error!("yt-dlp output: {}", first_line);
            first_line.to_string()
        } else {
            error!("yt-dlp failed with no output");
            "Command failed with unknown error".to_string()
        };

        return Err(YtDlpError::CommandFailed(error_msg));
    }

    // Log successful output
    let stdout = String::from_utf8_lossy(&output.stdout);
    trace!("yt-dlp stdout:\n{}", stdout);

    info!(
        "yt-dlp command executed successfully, searching for VTT file in {}",
        out_dir.display()
    );

    find_vtt_file(out_dir)
}

/// Find the most recently created VTT file in the output directory
fn find_vtt_file(dir: &Path) -> Result<PathBuf, YtDlpError> {
    debug!("Searching for VTT files in: {}", dir.display());

    let entries = std::fs::read_dir(dir).map_err(|e| {
        error!("Failed to read output directory {}: {}", dir.display(), e);
        YtDlpError::OutputDirReadFailed(e.to_string())
    })?;

    let mut vtt_files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            let is_vtt = entry.path().extension().map_or(false, |ext| ext == "vtt");
            if is_vtt {
                trace!("Found VTT file: {}", entry.path().display());
            }
            is_vtt
        })
        .collect();

    debug!("Found {} VTT file(s) in directory", vtt_files.len());

    if vtt_files.is_empty() {
        warn!("No VTT files found in output directory");
        return Err(YtDlpError::NoSubtitlesFound);
    }

    // Sort by modification time to get the most recent
    vtt_files.sort_by(|a, b| {
        let time_a = a.metadata().ok().and_then(|m| m.modified().ok());
        let time_b = b.metadata().ok().and_then(|m| m.modified().ok());
        time_b.cmp(&time_a) // Reverse order for most recent first
    });

    let vtt_file = vtt_files.first().map(|entry| entry.path()).ok_or_else(|| {
        error!("No VTT files found after filtering");
        YtDlpError::NoSubtitlesFound
    })?;

    info!("Selected VTT file: {}", vtt_file.display());

    // Log file size for debugging
    if let Ok(metadata) = std::fs::metadata(&vtt_file) {
        debug!("VTT file size: {} bytes", metadata.len());
    }

    Ok(vtt_file)
}

/// Remove existing VTT files from the output directory
pub fn cleanup_vtt_files(dir: &Path) {
    trace!("Starting VTT file cleanup in: {}", dir.display());

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            debug!("Could not read directory for cleanup: {}", e);
            return;
        }
    };

    let mut removed_count = 0;
    let mut failed_count = 0;

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "vtt") {
            trace!("Attempting to remove old VTT file: {}", path.display());

            match std::fs::remove_file(&path) {
                Ok(_) => {
                    removed_count += 1;
                    trace!("Removed VTT file: {}", path.display());
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Failed to remove old VTT file {}: {}", path.display(), e);
                }
            }
        }
    }

    if removed_count > 0 || failed_count > 0 {
        debug!(
            "VTT cleanup complete: {} removed, {} failed",
            removed_count, failed_count
        );
    } else {
        trace!("No VTT files to clean up");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_vtt_files_nonexistent_dir() {
        // Should not panic on non-existent directory
        let fake_dir = Path::new("/nonexistent/path/that/does/not/exist");
        cleanup_vtt_files(fake_dir);
    }

    #[test]
    fn test_find_vtt_file_empty_dir() {
        // Create a temp directory with no VTT files
        let temp_dir = std::env::temp_dir().join("yt_dlp_test_empty");
        let _ = std::fs::create_dir_all(&temp_dir);

        // Clean any existing files
        cleanup_vtt_files(&temp_dir);

        // Should return NoSubtitlesFound
        let result = find_vtt_file(&temp_dir);
        assert!(matches!(result, Err(YtDlpError::NoSubtitlesFound)));

        // Cleanup
        let _ = std::fs::remove_dir(&temp_dir);
    }
}
