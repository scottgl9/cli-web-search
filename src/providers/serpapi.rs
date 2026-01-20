//! SerpAPI provider
//!
//! SerpAPI provides search results from multiple search engines (Google, Bing, Yahoo, etc.)
//! See: https://serpapi.com/

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::cli::SafeSearch;
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const SERPAPI_BASE_URL: &str = "https://serpapi.com/search";

/// SerpAPI provider
pub struct SerpApiProvider {
    api_key: String,
    client: Client,
}

impl SerpApiProvider {
    /// Create a new SerpAPI provider with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for SerpApiProvider {
    fn name(&self) -> &'static str {
        "serpapi"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::missing_api_key(
                "serpapi",
                "CLI_WEB_SEARCH_SERPAPI_API_KEY",
            ));
        }

        let safe = match options.safe_search {
            SafeSearch::Off => "off",
            SafeSearch::Moderate => "medium",
            SafeSearch::Strict => "active",
        };

        // Build query parameters
        let mut params = vec![
            ("q", query.to_string()),
            ("api_key", self.api_key.clone()),
            ("engine", "google".to_string()),
            ("num", options.num_results.to_string()),
            ("safe", safe.to_string()),
        ];

        // Add date range filter if specified
        if let Some(ref date_range) = options.date_range {
            let tbs = match date_range {
                crate::cli::DateRange::Day => "qdr:d",
                crate::cli::DateRange::Week => "qdr:w",
                crate::cli::DateRange::Month => "qdr:m",
                crate::cli::DateRange::Year => "qdr:y",
            };
            params.push(("tbs", tbs.to_string()));
        }

        let response = self
            .client
            .get(SERPAPI_BASE_URL)
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
            return Err(SearchError::rate_limited("serpapi", retry_after));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("serpapi"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "serpapi",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let serpapi_response: SerpApiResponse = response.json().await?;

        // Check for API error in response
        if let Some(error) = serpapi_response.error {
            return Err(SearchError::api("serpapi", error));
        }

        let results = serpapi_response
            .organic_results
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.title,
                url: r.link,
                snippet: r.snippet.unwrap_or_default(),
                position: r.position.unwrap_or(i + 1),
                published_date: r.date,
                source: r.displayed_link.map(|l| extract_domain(&l)),
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        let params = [
            ("q", "test"),
            ("api_key", &self.api_key),
            ("engine", "google"),
            ("num", "1"),
        ];

        let response = self
            .client
            .get(SERPAPI_BASE_URL)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

/// Extract domain from displayed link
fn extract_domain(displayed_link: &str) -> String {
    // Displayed link is usually like "https://example.com › path"
    displayed_link
        .split(" › ")
        .next()
        .unwrap_or(displayed_link)
        .replace("https://", "")
        .replace("http://", "")
        .to_string()
}

// SerpAPI response structures

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SerpApiResponse {
    #[serde(default)]
    organic_results: Option<Vec<SerpApiResult>>,

    #[serde(default)]
    error: Option<String>,

    #[serde(default)]
    search_metadata: Option<SerpApiSearchMetadata>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SerpApiResult {
    title: String,
    link: String,
    #[serde(default)]
    snippet: Option<String>,
    #[serde(default)]
    position: Option<usize>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    displayed_link: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SerpApiSearchMetadata {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    total_time_taken: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serpapi_provider_not_configured() {
        let provider = SerpApiProvider::new(String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_serpapi_provider_configured() {
        let provider = SerpApiProvider::new("test-api-key".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "serpapi");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://example.com › path › page"),
            "example.com"
        );
        assert_eq!(extract_domain("http://simple.com"), "simple.com");
        assert_eq!(extract_domain("example.org › test"), "example.org");
    }

    #[test]
    fn test_extract_domain_no_separator() {
        assert_eq!(extract_domain("example.com"), "example.com");
    }

    #[test]
    fn test_extract_domain_empty() {
        assert_eq!(extract_domain(""), "");
    }

    #[test]
    fn test_serpapi_response_deserialization() {
        let json = r#"{
            "organic_results": [
                {
                    "title": "Test Result",
                    "link": "https://example.com",
                    "snippet": "This is a test snippet",
                    "position": 1,
                    "displayed_link": "https://example.com › page"
                }
            ],
            "search_metadata": {
                "id": "test-id",
                "status": "Success"
            }
        }"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        assert!(response.organic_results.is_some());
        let results = response.organic_results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Result");
        assert_eq!(results[0].link, "https://example.com");
    }

    #[test]
    fn test_serpapi_response_with_error() {
        let json = r#"{
            "error": "Invalid API key"
        }"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap(), "Invalid API key");
    }

    #[test]
    fn test_serpapi_empty_response() {
        let json = r#"{}"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        assert!(response.organic_results.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_serpapi_response_with_date() {
        let json = r#"{
            "organic_results": [
                {
                    "title": "Dated Result",
                    "link": "https://example.com",
                    "date": "2024-01-15"
                }
            ]
        }"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        let results = response.organic_results.unwrap();
        assert_eq!(results[0].date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_serpapi_response_minimal_result() {
        let json = r#"{
            "organic_results": [
                {
                    "title": "Minimal",
                    "link": "https://example.com"
                }
            ]
        }"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        let results = response.organic_results.unwrap();
        assert_eq!(results[0].title, "Minimal");
        assert!(results[0].snippet.is_none());
        assert!(results[0].position.is_none());
        assert!(results[0].date.is_none());
        assert!(results[0].displayed_link.is_none());
    }

    #[test]
    fn test_serpapi_response_multiple_results() {
        let json = r#"{
            "organic_results": [
                {
                    "title": "Result 1",
                    "link": "https://example1.com",
                    "position": 1
                },
                {
                    "title": "Result 2",
                    "link": "https://example2.com",
                    "position": 2
                },
                {
                    "title": "Result 3",
                    "link": "https://example3.com",
                    "position": 3
                }
            ]
        }"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        let results = response.organic_results.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].position, Some(1));
        assert_eq!(results[2].position, Some(3));
    }

    #[test]
    fn test_serpapi_search_metadata() {
        let json = r#"{
            "search_metadata": {
                "id": "abc123",
                "status": "Success",
                "total_time_taken": 1.5
            }
        }"#;

        let response: SerpApiResponse = serde_json::from_str(json).unwrap();
        assert!(response.search_metadata.is_some());
        let metadata = response.search_metadata.unwrap();
        assert_eq!(metadata.id, Some("abc123".to_string()));
        assert_eq!(metadata.status, Some("Success".to_string()));
        assert_eq!(metadata.total_time_taken, Some(1.5));
    }

    #[tokio::test]
    async fn test_serpapi_search_missing_api_key() {
        let provider = SerpApiProvider::new(String::new());
        let options = SearchOptions::default();

        let result = provider.search("test query", &options).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, SearchError::MissingApiKey { .. }));
        if let SearchError::MissingApiKey { provider, env_var } = error {
            assert_eq!(provider, "serpapi");
            assert_eq!(env_var, "CLI_WEB_SEARCH_SERPAPI_API_KEY");
        }
    }

    #[tokio::test]
    async fn test_serpapi_validate_api_key_not_configured() {
        let provider = SerpApiProvider::new(String::new());
        let result = provider.validate_api_key().await.unwrap();
        assert!(!result);
    }
}
