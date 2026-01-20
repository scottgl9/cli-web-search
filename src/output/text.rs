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
    let cleaned: String = text.split_whitespace().collect::<Vec<_>>().join(" ");

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

    fn create_test_result(title: &str, position: usize) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            url: format!("https://example.com/{}", position),
            snippet: format!("Snippet for {}", title),
            position,
            published_date: None,
            source: None,
        }
    }

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

    #[test]
    fn test_text_formatter_default() {
        let formatter = TextFormatter;
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let output = formatter.format(&response);
        assert!(output.contains("Search:"));
    }

    #[test]
    fn test_text_empty_results() {
        let response =
            SearchResponse::new("no results".to_string(), "google".to_string(), vec![], 50);

        let formatter = TextFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("No results found."));
    }

    #[test]
    fn test_text_multiple_results() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![
                create_test_result("First Result", 1),
                create_test_result("Second Result", 2),
                create_test_result("Third Result", 3),
            ],
            200,
        );

        let formatter = TextFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("1. First Result"));
        assert!(output.contains("2. Second Result"));
        assert!(output.contains("3. Third Result"));
        assert!(output.contains("3 results from brave"));
    }

    #[test]
    fn test_text_header_separator() {
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);

        let formatter = TextFormatter::new();
        let output = formatter.format(&response);

        // Should have a separator line of equals signs
        assert!(output.contains(&"=".repeat(60)));
    }

    #[test]
    fn test_truncate_whitespace_cleanup() {
        let messy_text = "This   has   extra   spaces   and\n\nnewlines";
        let result = truncate_snippet(messy_text, 100);

        // Should collapse multiple whitespace
        assert!(!result.contains("   "));
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_truncate_exact_length() {
        let text = "Exactly fifty characters long text for testing!!";
        let result = truncate_snippet(text, 48);

        // Should fit exactly without truncation
        assert_eq!(result, text);
    }

    #[test]
    fn test_text_search_time() {
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 12345);

        let formatter = TextFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("12345ms"));
    }

    #[test]
    fn test_text_url_indentation() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![create_test_result("Test", 1)],
            100,
        );

        let formatter = TextFormatter::new();
        let output = formatter.format(&response);

        // URLs should be indented with 3 spaces
        assert!(output.contains("   https://"));
    }
}
