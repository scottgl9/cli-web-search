//! Brave Search API provider

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::cli::SafeSearch;
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const BRAVE_API_URL: &str = "https://api.search.brave.com/res/v1/web/search";

/// Brave Search API provider
pub struct BraveProvider {
    api_key: String,
    client: Client,
}

impl BraveProvider {
    /// Create a new Brave provider with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for BraveProvider {
    fn name(&self) -> &'static str {
        "brave"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::missing_api_key(
                "brave",
                "CLI_WEB_SEARCH_BRAVE_API_KEY",
            ));
        }

        let safe_search = match options.safe_search {
            SafeSearch::Off => "off",
            SafeSearch::Moderate => "moderate",
            SafeSearch::Strict => "strict",
        };

        let mut request = self
            .client
            .get(BRAVE_API_URL)
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .query(&[
                ("q", query),
                ("count", &options.num_results.to_string()),
                ("safesearch", safe_search),
            ])
            .timeout(options.timeout);

        // Add freshness filter if date range specified
        if let Some(ref date_range) = options.date_range {
            let freshness = match date_range {
                crate::cli::DateRange::Day => "pd",
                crate::cli::DateRange::Week => "pw",
                crate::cli::DateRange::Month => "pm",
                crate::cli::DateRange::Year => "py",
            };
            request = request.query(&[("freshness", freshness)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if status == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            return Err(SearchError::rate_limited("brave", retry_after));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("brave"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "brave",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let brave_response: BraveSearchResponse = response.json().await?;

        let results = brave_response
            .web
            .map(|web| web.results)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.description.unwrap_or_default(),
                position: i + 1,
                published_date: r.age,
                source: r.meta_url.and_then(|m| m.hostname),
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        // Do a minimal search to validate the key
        let response = self
            .client
            .get(BRAVE_API_URL)
            .header("X-Subscription-Token", &self.api_key)
            .header("Accept", "application/json")
            .query(&[("q", "test"), ("count", "1")])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

// Brave API response structures

#[derive(Debug, Deserialize)]
struct BraveSearchResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: Option<String>,
    age: Option<String>,
    meta_url: Option<BraveMetaUrl>,
}

#[derive(Debug, Deserialize)]
struct BraveMetaUrl {
    hostname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brave_provider_not_configured() {
        let provider = BraveProvider::new(String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_brave_provider_configured() {
        let provider = BraveProvider::new("test-api-key".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "brave");
    }

    #[test]
    fn test_brave_response_deserialization() {
        let json = r#"{
            "web": {
                "results": [
                    {
                        "title": "Rust Programming Language",
                        "url": "https://www.rust-lang.org/",
                        "description": "A language empowering everyone to build reliable software.",
                        "age": "2024-01-15",
                        "meta_url": {
                            "hostname": "rust-lang.org"
                        }
                    }
                ]
            }
        }"#;

        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_some());
        let web = response.web.unwrap();
        assert_eq!(web.results.len(), 1);
        assert_eq!(web.results[0].title, "Rust Programming Language");
        assert_eq!(web.results[0].url, "https://www.rust-lang.org/");
    }

    #[test]
    fn test_brave_response_empty_results() {
        let json = r#"{
            "web": {
                "results": []
            }
        }"#;

        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_some());
        assert!(response.web.unwrap().results.is_empty());
    }

    #[test]
    fn test_brave_response_no_web() {
        let json = r#"{}"#;

        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_none());
    }

    #[test]
    fn test_brave_result_optional_fields() {
        let json = r#"{
            "web": {
                "results": [
                    {
                        "title": "Test",
                        "url": "https://example.com/"
                    }
                ]
            }
        }"#;

        let response: BraveSearchResponse = serde_json::from_str(json).unwrap();
        let result = &response.web.unwrap().results[0];
        assert!(result.description.is_none());
        assert!(result.age.is_none());
        assert!(result.meta_url.is_none());
    }

    #[tokio::test]
    async fn test_brave_search_missing_api_key() {
        let provider = BraveProvider::new(String::new());
        let options = SearchOptions::new();

        let result = provider.search("test", &options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::MissingApiKey { provider, .. } => {
                assert_eq!(provider, "brave");
            }
            _ => panic!("Expected MissingApiKey error"),
        }
    }

    #[tokio::test]
    async fn test_brave_validate_api_key_not_configured() {
        let provider = BraveProvider::new(String::new());
        let result = provider.validate_api_key().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
