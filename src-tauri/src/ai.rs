//! AI-powered transcript cleaning using the Groq API.
//!
//! This module provides functionality to clean and improve transcript text
//! using large language models via the Groq API.

use log::{debug, error, info, trace, warn};
use std::sync::Arc;

use reqwest::Client as ReqwestClient;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::errors::CaptionError;
use crate::settings;

/// Default Groq API base endpoint
const DEFAULT_ENDPOINT: &str = "https://api.groq.com/openai/v1";

/// Default model to use for AI cleaning
const DEFAULT_MODEL: &str = "llama-3.1-8b-instant";

/// Maximum chunk size in characters to avoid hitting token limits
const MAX_CHUNK_SIZE: usize = 8000;

const SYSTEM_PROMPT: &str = r#"
You are a transcript cleaner and light rewriter.

Your task is to:
- Fix spelling mistakes
- Fix grammar mistakes
- Fix punctuation
- Fix capitalization
- Fix spacing and broken line formatting
- Lightly rewrite to remove repeated phrases or redundant sentences

You MUST NOT:
- Change meaning in any way
- Add new facts or ideas
- Omit any essential information

Preserve the original language and tone as much as possible.
Only remove repetitions or obvious redundancies when it does not change content.
Merge lines only when they are clearly part of the same sentence.

Return ONLY the cleaned transcript text.
"#;

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

/// A reusable client for interacting with the Groq API.
///
/// This client maintains a connection pool and can be reused across multiple requests.
#[derive(Debug, Clone)]
pub struct GroqClient {
    api_key: String,
    client: Arc<ReqwestClient>,
    endpoint: String,
}

impl GroqClient {
    /// Creates a new GroqClient with the provided API key and optional custom endpoint.
    ///
    /// # Parameters
    ///
    /// - `api_key`: The Groq API key for authentication.
    /// - `endpoint`: Optional custom API endpoint. Defaults to `https://api.groq.com/openai/v1`.
    ///
    /// # Returns
    ///
    /// A new `GroqClient` instance.
    pub fn new(api_key: String, endpoint: Option<String>) -> Self {
        let ep = endpoint.unwrap_or_else(|| String::from(DEFAULT_ENDPOINT));
        info!("Creating new GroqClient with endpoint: {}", ep);
        debug!("API key configured (length: {} chars)", api_key.len());
        trace!(
            "API key prefix: {}...",
            &api_key.chars().take(8).collect::<String>()
        );

        Self {
            api_key,
            client: Arc::new(ReqwestClient::new()),
            endpoint: ep,
        }
    }

    /// Creates a new GroqClient from the `GROQ_API_KEY` environment variable.
    ///
    /// # Returns
    ///
    /// A `Result` containing the client or an error if the API key is not set.
    pub fn from_env() -> Result<Self, CaptionError> {
        trace!("Attempting to create GroqClient from environment");

        let api_key = std::env::var("GROQ_API_KEY").map_err(|_| {
            error!("GROQ_API_KEY environment variable not set");
            CaptionError::MissingApiKey("GROQ_API_KEY".into())
        })?;

        info!("Successfully loaded GROQ_API_KEY from environment");
        Ok(Self::new(api_key, None))
    }

    /// Creates a new GroqClient using the effective API key.
    ///
    /// This checks stored settings first, then falls back to environment variable.
    ///
    /// # Parameters
    ///
    /// - `app`: The Tauri app handle for accessing stored settings.
    ///
    /// # Returns
    ///
    /// A `Result` containing the client or an error if no API key is configured.
    pub fn from_app(app: &AppHandle) -> Result<Self, CaptionError> {
        trace!("Attempting to create GroqClient from app settings");

        let api_key = settings::get_effective_api_key(app).ok_or_else(|| {
            error!("No API key configured (checked store and environment)");
            CaptionError::MissingApiKey("GROQ_API_KEY".into())
        })?;

        info!("Successfully loaded API key");
        Ok(Self::new(api_key, None))
    }

    /// Sends a request to the Groq API with the provided JSON body.
    ///
    /// # Parameters
    ///
    /// - `body`: The JSON body to send in the request.
    /// - `path`: The API path to append to the base endpoint (e.g., "/chat/completions").
    ///
    /// # Returns
    ///
    /// The response from the Groq API.
    async fn send_request(
        &self,
        body: Value,
        path: &str,
    ) -> Result<reqwest::Response, CaptionError> {
        let url = format!("{}{}", self.endpoint, path);
        info!("Sending request to Groq API: {}", url);
        trace!(
            "Request body: {}",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        );

        debug!("Making POST request to {}", url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Network error while sending request to Groq: {}", e);
                debug!("Request URL: {}", url);
                CaptionError::NetworkError(e.to_string())
            })?;

        let status = response.status();
        debug!("Received response with status: {}", status);

        if status.is_success() {
            info!("Groq API request successful (status: {})", status);
        } else {
            warn!("Groq API request returned non-success status: {}", status);
        }

        Ok(response)
    }

    /// Sends a chat completion request to the Groq API.
    ///
    /// # Parameters
    ///
    /// - `model`: The model to use for the completion.
    /// - `messages`: The messages to send in the chat completion request.
    /// - `temperature`: The temperature for response randomness (0.0 = deterministic).
    ///
    /// # Returns
    ///
    /// The content of the assistant's response.
    pub async fn chat_completion(
        &self,
        model: &str,
        messages: Vec<Value>,
        temperature: f32,
    ) -> Result<String, CaptionError> {
        info!("Starting chat completion with model: {}", model);
        debug!(
            "Chat completion parameters: temperature={}, messages={}",
            temperature,
            messages.len()
        );
        trace!("Messages: {:?}", messages);

        let body = json!({
            "model": model,
            "temperature": temperature,
            "messages": messages,
        });

        let response = self.send_request(body, "/chat/completions").await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Groq API error (status {}): {}", status, body);
            return Err(CaptionError::ApiError(format!(
                "Groq API error ({}): {}",
                status, body
            )));
        }

        debug!("Parsing chat completion response");
        let completion: ChatCompletionResponse = response.json().await.map_err(|e| {
            error!("Failed to parse Groq API response: {}", e);
            CaptionError::ParseError(e.to_string())
        })?;

        let result = completion
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| {
                error!("Groq API returned empty choices array");
                CaptionError::ApiError("Empty response from Groq".into())
            })?;

        info!(
            "Chat completion successful: received {} characters",
            result.len()
        );
        trace!(
            "Response preview: {}...",
            &result.chars().take(100).collect::<String>()
        );

        Ok(result)
    }

    /// Cleans transcript text using AI.
    ///
    /// This method:
    /// - Fixes spelling, grammar, and punctuation
    /// - Preserves the original wording
    /// - Handles long texts by chunking if necessary
    ///
    /// # Parameters
    ///
    /// - `text`: The transcript text to clean.
    ///
    /// # Returns
    ///
    /// The cleaned transcript text.
    pub async fn clean_transcript(&self, text: &str) -> Result<String, CaptionError> {
        info!("Starting transcript cleaning: {} characters", text.len());

        if text.trim().is_empty() {
            debug!("Input text is empty, returning empty string");
            return Ok(String::new());
        }

        // If text is short enough, process in one request
        if text.len() <= MAX_CHUNK_SIZE {
            debug!(
                "Text fits in single chunk ({} <= {} chars), processing directly",
                text.len(),
                MAX_CHUNK_SIZE
            );
            return self.clean_chunk(text).await;
        }

        // For longer texts, split into chunks and process each
        info!(
            "Text exceeds max chunk size ({} > {} chars), splitting into chunks",
            text.len(),
            MAX_CHUNK_SIZE
        );
        let chunks = split_into_chunks(text, MAX_CHUNK_SIZE);
        info!("Split text into {} chunks", chunks.len());

        let mut results = Vec::with_capacity(chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            info!(
                "Processing chunk {}/{}: {} characters",
                i + 1,
                chunks.len(),
                chunk.len()
            );
            let cleaned = self.clean_chunk(chunk).await?;
            debug!(
                "Chunk {}/{} cleaned: {} -> {} chars",
                i + 1,
                chunks.len(),
                chunk.len(),
                cleaned.len()
            );
            results.push(cleaned);
        }

        let final_result = results.join("\n\n");
        info!(
            "All chunks processed: {} -> {} characters",
            text.len(),
            final_result.len()
        );

        Ok(final_result)
    }

    /// Cleans a single chunk of text.
    async fn clean_chunk(&self, text: &str) -> Result<String, CaptionError> {
        debug!("Cleaning chunk: {} characters", text.len());
        trace!(
            "Chunk content preview: {}...",
            &text.chars().take(100).collect::<String>()
        );

        let messages = vec![
            json!({
                "role": "system",
                "content": SYSTEM_PROMPT,
            }),
            json!({
                "role": "user",
                "content": text,
            }),
        ];

        trace!("Sending chunk to chat completion API");
        let result = self.chat_completion(DEFAULT_MODEL, messages, 0.0).await?;

        debug!(
            "Chunk cleaning complete: {} -> {} characters",
            text.len(),
            result.len()
        );

        Ok(result)
    }
}

/// Convenience function to clean transcript text using AI from environment config.
///
/// This creates a new GroqClient from the environment and cleans the text.
/// For multiple requests, prefer creating a `GroqClient` instance and reusing it.
///
/// # Parameters
///
/// - `text`: The transcript text to clean.
///
/// # Returns
///
/// The cleaned transcript text.
pub async fn clean_with_ai_from_env(text: &str) -> Result<String, CaptionError> {
    info!(
        "clean_with_ai_from_env called with {} characters",
        text.len()
    );
    trace!(
        "Input preview: {}...",
        &text.chars().take(100).collect::<String>()
    );

    debug!("Creating GroqClient from environment");
    let client = GroqClient::from_env()?;

    info!("Starting AI cleaning process");
    let result = client.clean_transcript(text).await?;

    info!(
        "AI cleaning complete: {} -> {} characters",
        text.len(),
        result.len()
    );

    Ok(result)
}

/// Convenience function to clean transcript text using AI from app settings.
///
/// This creates a new GroqClient using the effective API key (stored or env)
/// and cleans the text.
///
/// # Parameters
///
/// - `app`: The Tauri app handle for accessing stored settings.
/// - `text`: The transcript text to clean.
///
/// # Returns
///
/// The cleaned transcript text.
pub async fn clean_with_ai_from_app(app: &AppHandle, text: &str) -> Result<String, CaptionError> {
    info!(
        "clean_with_ai_from_app called with {} characters",
        text.len()
    );
    trace!(
        "Input preview: {}...",
        &text.chars().take(100).collect::<String>()
    );

    debug!("Creating GroqClient from app settings");
    let client = GroqClient::from_app(app)?;

    info!("Starting AI cleaning process");
    let result = client.clean_transcript(text).await?;

    info!(
        "AI cleaning complete: {} -> {} characters",
        text.len(),
        result.len()
    );

    Ok(result)
}

/// Splits text into chunks at paragraph boundaries.
fn split_into_chunks(text: &str, max_size: usize) -> Vec<String> {
    trace!(
        "Splitting text of {} chars into chunks of max {} chars",
        text.len(),
        max_size
    );

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for paragraph in text.split("\n\n") {
        // If adding this paragraph would exceed the limit, start a new chunk
        if !current_chunk.is_empty() && current_chunk.len() + paragraph.len() + 2 > max_size {
            trace!(
                "Starting new chunk (current: {} chars, paragraph: {} chars)",
                current_chunk.len(),
                paragraph.len()
            );
            chunks.push(current_chunk.trim().to_string());
            current_chunk = String::new();
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }
        current_chunk.push_str(paragraph);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }

    debug!("Split into {} chunks", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        trace!("Chunk {}: {} characters", i + 1, chunk.len());
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_groq_client_new() {
        let client = GroqClient::new("test-key".into(), None);
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.endpoint, DEFAULT_ENDPOINT);
    }

    #[test]
    fn test_groq_client_custom_endpoint() {
        let client = GroqClient::new("test-key".into(), Some("https://custom.api.com".into()));
        assert_eq!(client.endpoint, "https://custom.api.com");
    }

    #[test]
    fn test_split_into_chunks_small_text() {
        let text = "Hello world.\n\nThis is a test.";
        let chunks = split_into_chunks(text, 1000);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_split_into_chunks_large_text() {
        let paragraph = "This is a paragraph with some text. ".repeat(10);
        let text = format!("{}\n\n{}\n\n{}", paragraph, paragraph, paragraph);
        let chunks = split_into_chunks(&text, 500);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_split_into_chunks_empty() {
        let chunks = split_into_chunks("", 1000);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_into_chunks_single_large_paragraph() {
        let text = "a".repeat(1500);
        let chunks = split_into_chunks(&text, 1000);
        // Single paragraph that exceeds limit should still be in one chunk
        assert_eq!(chunks.len(), 1);
    }
}
