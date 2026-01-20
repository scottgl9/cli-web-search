//! Serper API provider
//!
//! Serper provides Google Search results via a simple API.
//! See: https://serper.dev/

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::cli::SafeSearch;
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const SERPER_API_URL: &str = "https://google.serper.dev/search";

/// Serper API provider
pub struct SerperProvider {
    api_key: String,
    client: Client,
}

impl SerperProvider {
    /// Create a new Serper provider with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for SerperProvider {
    fn name(&self) -> &'static str {
        "serper"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::missing_api_key(
                "serper",
                "CLI_WEB_SEARCH_SERPER_API_KEY",
            ));
        }

        let safe = match options.safe_search {
            SafeSearch::Off => false,
            SafeSearch::Moderate | SafeSearch::Strict => true,
        };

        let request_body = SerperRequest {
            q: query.to_string(),
            num: options.num_results,
            safe,
        };

        let response = self
            .client
            .post(SERPER_API_URL)
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
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
            return Err(SearchError::rate_limited("serper", retry_after));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("serper"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "serper",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let serper_response: SerperResponse = response.json().await?;

        let results = serper_response
            .organic
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.title,
                url: r.link,
                snippet: r.snippet.unwrap_or_default(),
                position: i + 1,
                published_date: r.date,
                source: extract_domain(&r.displayed_link),
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        let request_body = SerperRequest {
            q: "test".to_string(),
            num: 1,
            safe: false,
        };

        let response = self
            .client
            .post(SERPER_API_URL)
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

/// Extract domain from displayed link
fn extract_domain(displayed_link: &Option<String>) -> Option<String> {
    displayed_link.as_ref().and_then(|link| {
        // Displayed link is usually like "example.com › path"
        link.split(" › ").next().map(|s| s.to_string())
    })
}

// Serper API request/response structures

#[derive(Debug, Serialize)]
struct SerperRequest {
    q: String,
    num: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    safe: bool,
}

#[derive(Debug, Deserialize)]
struct SerperResponse {
    #[serde(default)]
    organic: Option<Vec<SerperResult>>,

    #[serde(default)]
    search_parameters: Option<SerperSearchParams>,
}

#[derive(Debug, Deserialize)]
struct SerperResult {
    title: String,
    link: String,
    #[serde(default)]
    snippet: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default, rename = "displayedLink")]
    displayed_link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerperSearchParams {
    #[serde(default)]
    q: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serper_provider_not_configured() {
        let provider = SerperProvider::new(String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_serper_provider_configured() {
        let provider = SerperProvider::new("test-api-key".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "serper");
    }

    #[test]
    fn test_serper_request_serialization() {
        let request = SerperRequest {
            q: "test query".to_string(),
            num: 10,
            safe: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test query"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain(&Some("example.com › path › page".to_string())),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain(&Some("simple.com".to_string())),
            Some("simple.com".to_string())
        );
        assert_eq!(extract_domain(&None), None);
    }
}
