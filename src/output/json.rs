//! JSON output formatter

use super::{OutputFormatter, SearchResponse};

/// JSON formatter for programmatic consumption
pub struct JsonFormatter {
    pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter with pretty printing enabled
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// Create a compact JSON formatter
    #[allow(dead_code)]
    pub fn compact() -> Self {
        Self { pretty: false }
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, response: &SearchResponse) -> String {
        if self.pretty {
            serde_json::to_string_pretty(response)
                .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize: {}\"}}", e))
        } else {
            serde_json::to_string(response)
                .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize: {}\"}}", e))
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
    fn test_json_formatter() {
        let response = SearchResponse::new(
            "test query".to_string(),
            "brave".to_string(),
            vec![create_test_result("Test Result", 1)],
            100,
        );

        let formatter = JsonFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("\"query\": \"test query\""));
        assert!(output.contains("\"provider\": \"brave\""));
        assert!(output.contains("\"title\": \"Test Result\""));
    }

    #[test]
    fn test_compact_json() {
        let response = SearchResponse::new("test".to_string(), "google".to_string(), vec![], 50);

        let formatter = JsonFormatter::compact();
        let output = formatter.format(&response);

        // Compact JSON should not have newlines in the main structure
        assert!(!output.contains("\n  \"query\""));
    }

    #[test]
    fn test_json_formatter_default() {
        let formatter = JsonFormatter::default();
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 100);
        let output = formatter.format(&response);
        // Default should be pretty printed
        assert!(output.contains('\n'));
    }

    #[test]
    fn test_json_multiple_results() {
        let response = SearchResponse::new(
            "query".to_string(),
            "brave".to_string(),
            vec![
                create_test_result("Result 1", 1),
                create_test_result("Result 2", 2),
                create_test_result("Result 3", 3),
            ],
            200,
        );

        let formatter = JsonFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("Result 1"));
        assert!(output.contains("Result 2"));
        assert!(output.contains("Result 3"));
        assert!(output.contains("\"results\""));
    }

    #[test]
    fn test_json_empty_results() {
        let response = SearchResponse::new(
            "no results query".to_string(),
            "tavily".to_string(),
            vec![],
            50,
        );

        let formatter = JsonFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("\"results\": []"));
        assert!(output.contains("no results query"));
    }

    #[test]
    fn test_json_search_time() {
        let response = SearchResponse::new("test".to_string(), "brave".to_string(), vec![], 12345);

        let formatter = JsonFormatter::new();
        let output = formatter.format(&response);

        assert!(output.contains("12345"));
        assert!(output.contains("search_time_ms"));
    }

    #[test]
    fn test_json_parseable() {
        let response = SearchResponse::new(
            "test query".to_string(),
            "brave".to_string(),
            vec![create_test_result("Test", 1)],
            100,
        );

        let formatter = JsonFormatter::new();
        let output = formatter.format(&response);

        // Verify the output is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["query"], "test query");
        assert_eq!(parsed["provider"], "brave");
        assert_eq!(parsed["results"].as_array().unwrap().len(), 1);
    }
}
