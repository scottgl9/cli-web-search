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
            serde_json::to_string_pretty(response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize: {}\"}}", e)
            })
        } else {
            serde_json::to_string(response).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize: {}\"}}", e)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::SearchResult;

    #[test]
    fn test_json_formatter() {
        let response = SearchResponse::new(
            "test query".to_string(),
            "brave".to_string(),
            vec![SearchResult {
                title: "Test Result".to_string(),
                url: "https://example.com".to_string(),
                snippet: "A test result".to_string(),
                position: 1,
                published_date: None,
                source: None,
            }],
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
        let response = SearchResponse::new(
            "test".to_string(),
            "google".to_string(),
            vec![],
            50,
        );

        let formatter = JsonFormatter::compact();
        let output = formatter.format(&response);

        // Compact JSON should not have newlines in the main structure
        assert!(!output.contains("\n  \"query\""));
    }
}
