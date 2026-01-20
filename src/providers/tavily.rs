//! Tavily Search API provider

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const TAVILY_API_URL: &str = "https://api.tavily.com/search";

/// Tavily Search API provider
pub struct TavilyProvider {
    api_key: String,
    client: Client,
}

impl TavilyProvider {
    /// Create a new Tavily provider with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for TavilyProvider {
    fn name(&self) -> &'static str {
        "tavily"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::missing_api_key(
                "tavily",
                "CLI_WEB_SEARCH_TAVILY_API_KEY",
            ));
        }

        let request_body = TavilySearchRequest {
            api_key: self.api_key.clone(),
            query: query.to_string(),
            max_results: options.num_results,
            include_domains: options.include_domains.clone().unwrap_or_default(),
            exclude_domains: options.exclude_domains.clone().unwrap_or_default(),
            search_depth: "basic".to_string(),
        };

        let response = self
            .client
            .post(TAVILY_API_URL)
            .json(&request_body)
            .timeout(options.timeout)
            .send()
            .await?;

        let status = response.status();
        if status == 429 {
            return Err(SearchError::rate_limited("tavily", None));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("tavily"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "tavily",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let tavily_response: TavilySearchResponse = response.json().await?;

        let results = tavily_response
            .results
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.content,
                position: i + 1,
                published_date: r.published_date,
                source: None,
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        let request_body = TavilySearchRequest {
            api_key: self.api_key.clone(),
            query: "test".to_string(),
            max_results: 1,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
            search_depth: "basic".to_string(),
        };

        let response = self
            .client
            .post(TAVILY_API_URL)
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

// Tavily API request/response structures

#[derive(Debug, Serialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    max_results: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    include_domains: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exclude_domains: Vec<String>,
    search_depth: String,
}

#[derive(Debug, Deserialize)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    #[serde(default)]
    published_date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tavily_provider_not_configured() {
        let provider = TavilyProvider::new(String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_tavily_provider_configured() {
        let provider = TavilyProvider::new("test-api-key".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "tavily");
    }

    #[test]
    fn test_tavily_response_deserialization() {
        let json = r#"{
            "results": [
                {
                    "title": "Rust Lang",
                    "url": "https://www.rust-lang.org/",
                    "content": "Rust is a systems programming language.",
                    "published_date": "2024-01-01"
                }
            ]
        }"#;

        let response: TavilySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Rust Lang");
        assert_eq!(response.results[0].url, "https://www.rust-lang.org/");
        assert_eq!(
            response.results[0].published_date,
            Some("2024-01-01".to_string())
        );
    }

    #[test]
    fn test_tavily_response_empty_results() {
        let json = r#"{
            "results": []
        }"#;

        let response: TavilySearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.results.is_empty());
    }

    #[test]
    fn test_tavily_response_no_published_date() {
        let json = r#"{
            "results": [
                {
                    "title": "Test",
                    "url": "https://example.com/",
                    "content": "Content"
                }
            ]
        }"#;

        let response: TavilySearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.results[0].published_date.is_none());
    }

    #[test]
    fn test_tavily_request_serialization() {
        let request = TavilySearchRequest {
            api_key: "test-key".to_string(),
            query: "rust programming".to_string(),
            max_results: 10,
            include_domains: vec!["rust-lang.org".to_string()],
            exclude_domains: vec![],
            search_depth: "basic".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test-key"));
        assert!(json.contains("rust programming"));
        assert!(json.contains("rust-lang.org"));
        // Empty exclude_domains should be skipped
        assert!(!json.contains("exclude_domains"));
    }

    #[test]
    fn test_tavily_request_empty_domains_skipped() {
        let request = TavilySearchRequest {
            api_key: "key".to_string(),
            query: "query".to_string(),
            max_results: 5,
            include_domains: vec![],
            exclude_domains: vec![],
            search_depth: "basic".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("include_domains"));
        assert!(!json.contains("exclude_domains"));
    }

    #[tokio::test]
    async fn test_tavily_search_missing_api_key() {
        let provider = TavilyProvider::new(String::new());
        let options = SearchOptions::new();

        let result = provider.search("test", &options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::MissingApiKey { provider, .. } => {
                assert_eq!(provider, "tavily");
            }
            _ => panic!("Expected MissingApiKey error"),
        }
    }

    #[tokio::test]
    async fn test_tavily_validate_api_key_not_configured() {
        let provider = TavilyProvider::new(String::new());
        let result = provider.validate_api_key().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
