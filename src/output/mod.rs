//! Output formatting for search results

mod json;
mod markdown;
mod text;

pub use self::json::JsonFormatter;
pub use self::markdown::MarkdownFormatter;
pub use self::text::TextFormatter;

use crate::cli::OutputFormat;
use crate::providers::SearchResult;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Metadata about the search
#[derive(Debug, Clone, Serialize)]
pub struct SearchMetadata {
    /// The original query
    pub query: String,

    /// The provider used
    pub provider: String,

    /// Timestamp of the search
    pub timestamp: DateTime<Utc>,

    /// Number of results returned
    pub total_results: usize,

    /// Search time in milliseconds
    pub search_time_ms: u64,
}

/// Complete search response with metadata
#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    /// Search metadata
    #[serde(flatten)]
    pub metadata: SearchMetadata,

    /// Search results
    pub results: Vec<SearchResult>,
}

impl SearchResponse {
    pub fn new(
        query: String,
        provider: String,
        results: Vec<SearchResult>,
        search_time_ms: u64,
    ) -> Self {
        Self {
            metadata: SearchMetadata {
                query,
                provider,
                timestamp: Utc::now(),
                total_results: results.len(),
                search_time_ms,
            },
            results,
        }
    }
}

/// Trait for output formatters
pub trait OutputFormatter {
    /// Format the search response
    fn format(&self, response: &SearchResponse) -> String;
}

/// Get the appropriate formatter for the given output format
pub fn get_formatter(format: &OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Json => Box::new(JsonFormatter::new()),
        OutputFormat::Markdown => Box::new(MarkdownFormatter::new()),
        OutputFormat::Text => Box::new(TextFormatter::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_get_formatter_json() {
        let formatter = get_formatter(&OutputFormat::Json);
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let output = formatter.format(&response);
        // JSON output should contain curly braces
        assert!(output.starts_with("{"));
        assert!(output.contains("\"query\""));
    }

    #[test]
    fn test_get_formatter_markdown() {
        let formatter = get_formatter(&OutputFormat::Markdown);
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let output = formatter.format(&response);
        // Markdown output should start with a header
        assert!(output.starts_with("# Search Results:"));
    }

    #[test]
    fn test_get_formatter_text() {
        let formatter = get_formatter(&OutputFormat::Text);
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let output = formatter.format(&response);
        // Text output should start with "Search:"
        assert!(output.starts_with("Search:"));
    }

    #[test]
    fn test_search_response_new() {
        let results = vec![create_test_result("Test", 1)];
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            results.clone(),
            150,
        );

        assert_eq!(response.metadata.query, "query");
        assert_eq!(response.metadata.provider, "brave");
        assert_eq!(response.metadata.total_results, 1);
        assert_eq!(response.metadata.search_time_ms, 150);
        assert_eq!(response.results.len(), 1);
    }

    #[test]
    fn test_search_response_serialization() {
        let response = SearchResponse::new(
            "test query".to_string(),
            "google".to_string(),
            vec![create_test_result("Result", 1)],
            200,
        );

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"query\":\"test query\""));
        assert!(json.contains("\"provider\":\"google\""));
        assert!(json.contains("\"total_results\":1"));
    }

    #[test]
    fn test_search_metadata_serialization() {
        let metadata = SearchMetadata {
            query: "test".to_string(),
            provider: "brave".to_string(),
            timestamp: Utc::now(),
            total_results: 5,
            search_time_ms: 100,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"query\":\"test\""));
        assert!(json.contains("\"search_time_ms\":100"));
    }

    #[test]
    fn test_search_response_empty_results() {
        let response = SearchResponse::new("empty".to_string(), "tavily".to_string(), vec![], 50);

        assert_eq!(response.metadata.total_results, 0);
        assert!(response.results.is_empty());
    }

    #[test]
    fn test_search_response_debug() {
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let debug = format!("{:?}", response);
        assert!(debug.contains("SearchResponse"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_search_metadata_debug() {
        let metadata = SearchMetadata {
            query: "test".to_string(),
            provider: "brave".to_string(),
            timestamp: Utc::now(),
            total_results: 0,
            search_time_ms: 0,
        };
        let debug = format!("{:?}", metadata);
        assert!(debug.contains("SearchMetadata"));
    }
}
