use log::{debug, trace};
use regex::Regex;
use std::sync::LazyLock;

/// Pre-compiled regex for stripping HTML tags from VTT content.
static HTML_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]+>").expect("Invalid HTML tag regex"));

/// Pre-compiled regex for matching VTT timestamp lines.
static TIMESTAMP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{2}:\d{2}[:\.]").expect("Invalid timestamp regex"));

/// Checks if a line is a VTT header line that should be skipped.
#[inline]
fn is_header_line(line: &str) -> bool {
    let is_header = line.starts_with("WEBVTT")
        || line.starts_with("Kind:")
        || line.starts_with("Language:")
        || line.starts_with("NOTE");

    if is_header {
        trace!("Skipping header line: {}", line);
    }

    is_header
}

/// Checks if a line is a VTT timestamp/cue timing line.
#[inline]
fn is_timestamp_line(line: &str) -> bool {
    let is_timestamp = line.contains("-->") || TIMESTAMP_RE.is_match(line);

    if is_timestamp {
        trace!("Skipping timestamp line: {}", line);
    }

    is_timestamp
}

/// Checks if a line is a cue identifier (numeric or alphanumeric ID before timestamp).
#[inline]
fn is_cue_identifier(line: &str) -> bool {
    // Cue identifiers are typically just numbers or simple alphanumeric strings
    let is_cue_id = !line.is_empty()
        && line
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && line.parse::<u64>().is_ok();

    if is_cue_id {
        trace!("Skipping cue identifier: {}", line);
    }

    is_cue_id
}

/// Strips HTML tags and VTT-specific formatting from a line.
fn clean_line(line: &str) -> String {
    let cleaned = HTML_TAG_RE.replace_all(line, "");
    let result = cleaned.trim().to_string();

    if result != line.trim() {
        trace!("Cleaned line: '{}' -> '{}'", line.trim(), result);
    }

    result
}

/// Converts VTT subtitle content to plain text.
///
/// This function handles YouTube's "rolling" caption format where each cue
/// contains overlapping text from previous cues (to create a scrolling effect).
///
/// This function:
/// - Removes VTT headers and metadata
/// - Strips timestamp lines
/// - Removes HTML formatting tags
/// - Deduplicates individual lines (not just entire blocks) to handle YouTube's rolling captions
/// - Joins text segments appropriately
///
/// # Arguments
/// * `input` - The raw VTT file content as a string
///
/// # Returns
/// A cleaned plain text string with the subtitle content
pub fn vtt_to_text(input: &str) -> String {
    debug!(
        "Starting VTT to text conversion, input size: {} bytes",
        input.len()
    );
    trace!("Input line count: {}", input.lines().count());

    // Collect all unique lines in order of first appearance
    let mut unique_lines: Vec<String> = Vec::new();
    let mut seen_lines: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut lines_processed = 0;
    let mut lines_skipped = 0;
    let mut duplicates_removed = 0;

    for line in input.lines() {
        lines_processed += 1;
        let line = line.trim();

        // Skip empty lines, headers, timestamps, and cue identifiers
        if line.is_empty()
            || is_header_line(line)
            || is_timestamp_line(line)
            || is_cue_identifier(line)
        {
            lines_skipped += 1;
            continue;
        }

        let cleaned = clean_line(line);
        if cleaned.is_empty() {
            lines_skipped += 1;
            continue;
        }

        // Check if we've seen this exact line before
        if seen_lines.contains(&cleaned) {
            duplicates_removed += 1;
            trace!("Duplicate line removed: '{}'", cleaned);
            continue;
        }

        // New unique line - add it
        trace!("Adding unique line: '{}'", cleaned);
        seen_lines.insert(cleaned.clone());
        unique_lines.push(cleaned);
    }

    // Join all unique lines with spaces to form continuous text
    // Then normalize whitespace
    let result = unique_lines.join(" ");

    // Normalize multiple spaces to single space
    let result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    debug!(
        "VTT conversion complete: {} lines processed, {} skipped, {} duplicates removed",
        lines_processed, lines_skipped, duplicates_removed
    );
    debug!(
        "Output: {} unique text segments, {} characters, {} words",
        unique_lines.len(),
        result.len(),
        result.split_whitespace().count()
    );

    if !result.is_empty() {
        trace!(
            "Output preview: {}...",
            result.chars().take(100).collect::<String>()
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vtt_parsing() {
        let input = r#"WEBVTT
Kind: captions
Language: en

00:00:00.000 --> 00:00:02.000
Hello world

00:00:02.000 --> 00:00:04.000
This is a test
"#;
        let result = vtt_to_text(input);
        assert_eq!(result, "Hello world This is a test");
    }

    #[test]
    fn test_html_tag_removal() {
        let input = r#"WEBVTT

00:00:00.000 --> 00:00:02.000
<c.colorWhite>Hello</c> <b>world</b>
"#;
        let result = vtt_to_text(input);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_youtube_rolling_captions() {
        // This simulates YouTube's rolling caption format where lines repeat
        let input = r#"WEBVTT

00:00:00.000 --> 00:00:02.000
hello everyone welcome to another

00:00:02.000 --> 00:00:04.000
hello everyone welcome to another
another module in this online course

00:00:04.000 --> 00:00:06.000
another module in this online course
strategy and introduction to Game
"#;
        let result = vtt_to_text(input);
        assert_eq!(
            result,
            "hello everyone welcome to another another module in this online course strategy and introduction to Game"
        );
    }

    #[test]
    fn test_deduplication() {
        let input = r#"WEBVTT

00:00:00.000 --> 00:00:02.000
Hello world

00:00:01.000 --> 00:00:03.000
Hello world

00:00:02.000 --> 00:00:04.000
Goodbye world
"#;
        let result = vtt_to_text(input);
        assert_eq!(result, "Hello world Goodbye world");
    }

    #[test]
    fn test_multiline_cues() {
        let input = r#"WEBVTT

00:00:00.000 --> 00:00:02.000
Line one
Line two
"#;
        let result = vtt_to_text(input);
        assert_eq!(result, "Line one Line two");
    }

    #[test]
    fn test_cue_identifiers() {
        let input = r#"WEBVTT

1
00:00:00.000 --> 00:00:02.000
First cue

2
00:00:02.000 --> 00:00:04.000
Second cue
"#;
        let result = vtt_to_text(input);
        assert_eq!(result, "First cue Second cue");
    }

    #[test]
    fn test_empty_input() {
        let result = vtt_to_text("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_header_only() {
        let input = "WEBVTT\nKind: captions\nLanguage: en\n";
        let result = vtt_to_text(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_triple_repetition() {
        // Simulates the exact issue reported: lines appearing 3 times
        let input = r#"WEBVTT

00:00:00.000 --> 00:00:02.000
hello everyone welcome to another

00:00:01.000 --> 00:00:03.000
hello everyone welcome to another
another module in this online course

00:00:02.000 --> 00:00:04.000
hello everyone welcome to another
another module in this online course
strategy and introduction

00:00:03.000 --> 00:00:05.000
another module in this online course
strategy and introduction
Theory and uh
"#;
        let result = vtt_to_text(input);
        // Each unique line should appear only once
        assert_eq!(
            result,
            "hello everyone welcome to another another module in this online course strategy and introduction Theory and uh"
        );
    }

    #[test]
    fn test_whitespace_normalization() {
        let input = r#"WEBVTT

00:00:00.000 --> 00:00:02.000
Hello    world

00:00:02.000 --> 00:00:04.000
with   extra   spaces
"#;
        let result = vtt_to_text(input);
        // Multiple spaces should be normalized to single spaces
        assert_eq!(result, "Hello world with extra spaces");
    }
}
