//! Bing Web Search API provider
//!
//! Bing Web Search API provides web search results from Microsoft Bing.
//! See: https://docs.microsoft.com/en-us/bing/search-apis/bing-web-search/

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::cli::SafeSearch;
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const BING_API_URL: &str = "https://api.bing.microsoft.com/v7.0/search";

/// Bing Web Search API provider
pub struct BingProvider {
    api_key: String,
    client: Client,
}

impl BingProvider {
    /// Create a new Bing provider with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for BingProvider {
    fn name(&self) -> &'static str {
        "bing"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::missing_api_key(
                "bing",
                "CLI_WEB_SEARCH_BING_API_KEY",
            ));
        }

        let safe_search = match options.safe_search {
            SafeSearch::Off => "Off",
            SafeSearch::Moderate => "Moderate",
            SafeSearch::Strict => "Strict",
        };

        // Build query parameters
        let mut params = vec![
            ("q", query.to_string()),
            ("count", options.num_results.to_string()),
            ("safeSearch", safe_search.to_string()),
            ("textFormat", "Raw".to_string()),
        ];

        // Add freshness filter for date range
        if let Some(ref date_range) = options.date_range {
            let freshness = match date_range {
                crate::cli::DateRange::Day => "Day",
                crate::cli::DateRange::Week => "Week",
                crate::cli::DateRange::Month => "Month",
                crate::cli::DateRange::Year => "Year",
            };
            params.push(("freshness", freshness.to_string()));
        }

        let response = self
            .client
            .get(BING_API_URL)
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .query(&params)
            .timeout(options.timeout)
            .send()
            .await?;

        let status = response.status();
        if status == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            return Err(SearchError::rate_limited("bing", retry_after));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("bing"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "bing",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let bing_response: BingResponse = response.json().await?;

        let results = bing_response
            .web_pages
            .map(|wp| wp.value)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.name,
                url: r.url,
                snippet: r.snippet.unwrap_or_default(),
                position: i + 1,
                published_date: r.date_last_crawled,
                source: Some(extract_domain(&r.display_url)),
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        let params = [("q", "test"), ("count", "1")];

        let response = self
            .client
            .get(BING_API_URL)
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

/// Extract domain from display URL
fn extract_domain(display_url: &str) -> String {
    // Display URL is usually like "example.com/path" or "www.example.com/path"
    display_url
        .split('/')
        .next()
        .unwrap_or(display_url)
        .to_string()
}

// Bing API response structures

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BingResponse {
    #[serde(rename = "webPages")]
    web_pages: Option<BingWebPages>,

    #[serde(rename = "_type")]
    response_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BingWebPages {
    #[serde(rename = "totalEstimatedMatches")]
    total_estimated_matches: Option<u64>,

    value: Vec<BingWebResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BingWebResult {
    name: String,
    url: String,
    #[serde(default)]
    snippet: Option<String>,
    #[serde(rename = "displayUrl")]
    display_url: String,
    #[serde(rename = "dateLastCrawled")]
    date_last_crawled: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bing_provider_not_configured() {
        let provider = BingProvider::new(String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_bing_provider_configured() {
        let provider = BingProvider::new("test-api-key".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "bing");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("example.com/path/page"), "example.com");
        assert_eq!(extract_domain("www.example.org/test"), "www.example.org");
        assert_eq!(extract_domain("simple.com"), "simple.com");
    }

    #[test]
    fn test_extract_domain_empty() {
        assert_eq!(extract_domain(""), "");
    }

    #[test]
    fn test_extract_domain_no_path() {
        assert_eq!(extract_domain("example.com"), "example.com");
    }

    #[test]
    fn test_bing_response_deserialization() {
        let json = r#"{
            "_type": "SearchResponse",
            "webPages": {
                "totalEstimatedMatches": 12345,
                "value": [
                    {
                        "name": "Test Result",
                        "url": "https://example.com/page",
                        "snippet": "This is a test snippet",
                        "displayUrl": "example.com/page",
                        "dateLastCrawled": "2024-01-15T12:00:00Z"
                    }
                ]
            }
        }"#;

        let response: BingResponse = serde_json::from_str(json).unwrap();
        assert!(response.web_pages.is_some());
        let web_pages = response.web_pages.unwrap();
        assert_eq!(web_pages.value.len(), 1);
        assert_eq!(web_pages.value[0].name, "Test Result");
        assert_eq!(web_pages.value[0].url, "https://example.com/page");
    }

    #[test]
    fn test_bing_empty_response() {
        let json = r#"{
            "_type": "SearchResponse"
        }"#;

        let response: BingResponse = serde_json::from_str(json).unwrap();
        assert!(response.web_pages.is_none());
    }

    #[test]
    fn test_bing_response_with_minimal_result() {
        let json = r#"{
            "webPages": {
                "value": [
                    {
                        "name": "Minimal Result",
                        "url": "https://example.com",
                        "displayUrl": "example.com"
                    }
                ]
            }
        }"#;

        let response: BingResponse = serde_json::from_str(json).unwrap();
        let web_pages = response.web_pages.unwrap();
        assert_eq!(web_pages.value[0].snippet, None);
        assert_eq!(web_pages.value[0].date_last_crawled, None);
    }

    #[test]
    fn test_bing_response_multiple_results() {
        let json = r#"{
            "webPages": {
                "totalEstimatedMatches": 1000,
                "value": [
                    {
                        "name": "Result 1",
                        "url": "https://example1.com",
                        "displayUrl": "example1.com"
                    },
                    {
                        "name": "Result 2",
                        "url": "https://example2.com",
                        "snippet": "Second result",
                        "displayUrl": "example2.com/path"
                    }
                ]
            }
        }"#;

        let response: BingResponse = serde_json::from_str(json).unwrap();
        let web_pages = response.web_pages.unwrap();
        assert_eq!(web_pages.value.len(), 2);
        assert_eq!(web_pages.total_estimated_matches, Some(1000));
    }

    #[tokio::test]
    async fn test_bing_search_missing_api_key() {
        let provider = BingProvider::new(String::new());
        let options = SearchOptions::default();

        let result = provider.search("test query", &options).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, SearchError::MissingApiKey { .. }));
        if let SearchError::MissingApiKey { provider, env_var } = error {
            assert_eq!(provider, "bing");
            assert_eq!(env_var, "CLI_WEB_SEARCH_BING_API_KEY");
        }
    }

    #[tokio::test]
    async fn test_bing_validate_api_key_not_configured() {
        let provider = BingProvider::new(String::new());
        let result = provider.validate_api_key().await.unwrap();
        assert!(!result);
    }
}
