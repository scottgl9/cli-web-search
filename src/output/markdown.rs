//! Markdown output formatter

use super::{OutputFormatter, SearchResponse};

/// Markdown formatter for human-readable output
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for MarkdownFormatter {
    fn format(&self, response: &SearchResponse) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "# Search Results: {}\n\n",
            response.metadata.query
        ));

        // Metadata line
        output.push_str(&format!(
            "*Provider: {} | Results: {} | Time: {}ms*\n\n",
            response.metadata.provider,
            response.metadata.total_results,
            response.metadata.search_time_ms
        ));

        output.push_str("---\n\n");

        // Results
        if response.results.is_empty() {
            output.push_str("*No results found.*\n");
        } else {
            for result in &response.results {
                // Title with position
                output.push_str(&format!("## {}. {}\n\n", result.position, result.title));

                // URL
                output.push_str(&format!("**URL:** {}\n\n", result.url));

                // Source/domain if available
                if let Some(ref source) = result.source {
                    output.push_str(&format!("**Source:** {}\n\n", source));
                }

                // Published date if available
                if let Some(ref date) = result.published_date {
                    output.push_str(&format!("**Published:** {}\n\n", date));
                }

                // Snippet
                if !result.snippet.is_empty() {
                    output.push_str(&format!("{}\n\n", result.snippet));
                }

                output.push_str("---\n\n");
            }
        }

        output
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
    fn test_markdown_formatter() {
        let response = SearchResponse::new(
            "rust programming".to_string(),
            "brave".to_string(),
            vec![
                SearchResult {
                    title: "The Rust Programming Language".to_string(),
                    url: "https://www.rust-lang.org".to_string(),
                    snippet: "A language empowering everyone to build reliable software."
                        .to_string(),
                    position: 1,
                    published_date: None,
                    source: Some("rust-lang.org".to_string()),
                },
                SearchResult {
                    title: "Rust Documentation".to_string(),
                    url: "https://doc.rust-lang.org".to_string(),
                    snippet: "Official Rust documentation.".to_string(),
                    position: 2,
                    published_date: None,
                    source: None,
                },
            ],
            150,
        );

        let formatter = MarkdownFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("# Search Results: rust programming"));
        assert!(output.contains("*Provider: brave | Results: 2 | Time: 150ms*"));
        assert!(output.contains("## 1. The Rust Programming Language"));
        assert!(output.contains("**URL:** https://www.rust-lang.org"));
        assert!(output.contains("**Source:** rust-lang.org"));
    }

    #[test]
    fn test_markdown_empty_results() {
        let response = SearchResponse::new(
            "nonexistent query".to_string(),
            "google".to_string(),
            vec![],
            50,
        );

        let formatter = MarkdownFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("*No results found.*"));
    }

    #[test]
    fn test_markdown_default() {
        let formatter = MarkdownFormatter;
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let output = formatter.format(&response);
        assert!(output.contains("# Search Results:"));
    }

    #[test]
    fn test_markdown_with_published_date() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![SearchResult {
                title: "Article".to_string(),
                url: "https://example.com".to_string(),
                snippet: "Content".to_string(),
                position: 1,
                published_date: Some("2024-01-15".to_string()),
                source: None,
            }],
            100,
        );

        let formatter = MarkdownFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("**Published:** 2024-01-15"));
    }

    #[test]
    fn test_markdown_separators() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![
                create_test_result("Result 1", 1),
                create_test_result("Result 2", 2),
            ],
            100,
        );

        let formatter = MarkdownFormatter::new();
        let output = formatter.format(&response);

        // Should have separator between results
        let separator_count = output.matches("---").count();
        assert!(separator_count >= 2); // At least one after header and one between/after results
    }

    #[test]
    fn test_markdown_result_positions() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![
                create_test_result("First", 1),
                create_test_result("Second", 2),
                create_test_result("Third", 3),
            ],
            100,
        );

        let formatter = MarkdownFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("## 1. First"));
        assert!(output.contains("## 2. Second"));
        assert!(output.contains("## 3. Third"));
    }

    #[test]
    fn test_markdown_url_formatting() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![SearchResult {
                title: "Test".to_string(),
                url: "https://example.com/path?query=value".to_string(),
                snippet: "Content".to_string(),
                position: 1,
                published_date: None,
                source: None,
            }],
            100,
        );

        let formatter = MarkdownFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("**URL:** https://example.com/path?query=value"));
    }
}
