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
