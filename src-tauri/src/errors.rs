use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptionError {
    #[error("Failed to create temp directory: {0}")]
    TempDirCreation(#[from] std::io::Error),

    #[error("Failed to download captions: {0}")]
    Download(#[from] crate::yt_dlp::YtDlpError),

    #[error("Failed to read VTT file: {0}")]
    VttRead(String),

    #[error("AI cleaning failed: {0}")]
    GroqError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Pipeline error: {0}")]
    PipelineError(String),

    #[error("Missing API key: {0}")]
    MissingApiKey(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

impl serde::Serialize for CaptionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
