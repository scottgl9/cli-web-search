//! Plain text output formatter

use super::{OutputFormatter, SearchResponse};

/// Plain text formatter for simple terminal output
pub struct TextFormatter;

impl TextFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for TextFormatter {
    fn format(&self, response: &SearchResponse) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "Search: \"{}\" ({} results from {} in {}ms)\n",
            response.metadata.query,
            response.metadata.total_results,
            response.metadata.provider,
            response.metadata.search_time_ms
        ));
        output.push_str(&"=".repeat(60));
        output.push('\n');
        output.push('\n');

        // Results
        if response.results.is_empty() {
            output.push_str("No results found.\n");
        } else {
            for result in &response.results {
                // Position and title
                output.push_str(&format!("{}. {}\n", result.position, result.title));

                // URL
                output.push_str(&format!("   {}\n", result.url));

                // Snippet (wrapped/truncated for readability)
                if !result.snippet.is_empty() {
                    let snippet = truncate_snippet(&result.snippet, 200);
                    output.push_str(&format!("   {}\n", snippet));
                }

                output.push('\n');
            }
        }

        output
    }
}

/// Truncate a snippet to a maximum length, adding ellipsis if needed
fn truncate_snippet(text: &str, max_len: usize) -> String {
    // Clean up whitespace
    let cleaned: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.len() <= max_len {
        cleaned
    } else {
        // Find a good break point
        let truncated = &cleaned[..max_len];
        if let Some(last_space) = truncated.rfind(' ') {
            format!("{}...", &truncated[..last_space])
        } else {
            format!("{}...", truncated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::SearchResult;

    #[test]
    fn test_text_formatter() {
        let response = SearchResponse::new(
            "rust programming".to_string(),
            "brave".to_string(),
            vec![SearchResult {
                title: "The Rust Programming Language".to_string(),
                url: "https://www.rust-lang.org".to_string(),
                snippet: "A language empowering everyone.".to_string(),
                position: 1,
                published_date: None,
                source: None,
            }],
            100,
        );

        let formatter = TextFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("Search: \"rust programming\""));
        assert!(output.contains("1 results from brave"));
        assert!(output.contains("1. The Rust Programming Language"));
        assert!(output.contains("https://www.rust-lang.org"));
    }

    #[test]
    fn test_truncate_snippet() {
        let long_text = "This is a very long snippet that should be truncated to fit within the specified maximum length for better readability in the terminal output.";
        let truncated = truncate_snippet(long_text, 50);

        assert!(truncated.len() <= 53); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_truncate_short_snippet() {
        let short_text = "Short snippet";
        let result = truncate_snippet(short_text, 50);

        assert_eq!(result, "Short snippet");
        assert!(!result.ends_with("..."));
    }
}
