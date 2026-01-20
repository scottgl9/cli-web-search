//! Firecrawl Search API provider

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const FIRECRAWL_API_URL: &str = "https://api.firecrawl.dev/v2/search";

/// Firecrawl Search API provider
pub struct FirecrawlProvider {
    api_key: String,
    client: Client,
}

impl FirecrawlProvider {
    /// Create a new Firecrawl provider with the given API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for FirecrawlProvider {
    fn name(&self) -> &'static str {
        "firecrawl"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::missing_api_key(
                "firecrawl",
                "CLI_WEB_SEARCH_FIRECRAWL_API_KEY",
            ));
        }

        // Build the request body
        let mut request_body = FirecrawlSearchRequest {
            query: query.to_string(),
            limit: options.num_results,
            sources: vec!["web".to_string()],
            tbs: None,
            country: Some("US".to_string()),
            timeout: Some((options.timeout.as_millis() as u64).min(60000)),
        };

        // Add time-based search filter if date range specified
        if let Some(ref date_range) = options.date_range {
            let tbs = match date_range {
                crate::cli::DateRange::Day => "qdr:d",
                crate::cli::DateRange::Week => "qdr:w",
                crate::cli::DateRange::Month => "qdr:m",
                crate::cli::DateRange::Year => "qdr:y",
            };
            request_body.tbs = Some(tbs.to_string());
        }

        // Build site: query modifiers for domain filtering
        let mut modified_query = query.to_string();
        if let Some(ref include_domains) = options.include_domains {
            for domain in include_domains {
                modified_query = format!("{} site:{}", modified_query, domain);
            }
        }
        if let Some(ref exclude_domains) = options.exclude_domains {
            for domain in exclude_domains {
                modified_query = format!("{} -site:{}", modified_query, domain);
            }
        }
        request_body.query = modified_query;

        let response = self
            .client
            .post(FIRECRAWL_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(options.timeout)
            .send()
            .await?;

        let status = response.status();
        if status == 429 {
            return Err(SearchError::rate_limited("firecrawl", None));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("firecrawl"));
        }

        if status == 408 {
            return Err(SearchError::Timeout(options.timeout.as_secs()));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "firecrawl",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let firecrawl_response: FirecrawlSearchResponse = response.json().await?;

        if !firecrawl_response.success {
            return Err(SearchError::api(
                "firecrawl",
                firecrawl_response
                    .warning
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let results = firecrawl_response
            .data
            .web
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, r)| SearchResult {
                title: r.title.unwrap_or_default(),
                url: r.url,
                snippet: r.description.unwrap_or_default(),
                position: i + 1,
                published_date: None,
                source: r.metadata.and_then(|m| m.source_url),
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        let request_body = FirecrawlSearchRequest {
            query: "test".to_string(),
            limit: 1,
            sources: vec!["web".to_string()],
            tbs: None,
            country: Some("US".to_string()),
            timeout: Some(10000),
        };

        let response = self
            .client
            .post(FIRECRAWL_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

// Firecrawl API request/response structures

#[derive(Debug, Serialize)]
struct FirecrawlSearchRequest {
    query: String,
    limit: usize,
    sources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tbs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirecrawlSearchResponse {
    success: bool,
    data: FirecrawlData,
    #[serde(default)]
    warning: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default, rename = "creditsUsed")]
    credits_used: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirecrawlData {
    #[serde(default)]
    web: Option<Vec<FirecrawlWebResult>>,
    #[serde(default)]
    images: Option<Vec<FirecrawlImageResult>>,
    #[serde(default)]
    news: Option<Vec<FirecrawlNewsResult>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirecrawlWebResult {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    url: String,
    #[serde(default)]
    markdown: Option<String>,
    #[serde(default)]
    metadata: Option<FirecrawlMetadata>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirecrawlImageResult {
    #[serde(default)]
    title: Option<String>,
    #[serde(default, rename = "imageUrl")]
    image_url: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    position: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirecrawlNewsResult {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    snippet: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    position: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FirecrawlMetadata {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "sourceURL")]
    source_url: Option<String>,
    #[serde(default, rename = "statusCode")]
    status_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firecrawl_provider_not_configured() {
        let provider = FirecrawlProvider::new(String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_firecrawl_provider_configured() {
        let provider = FirecrawlProvider::new("fc-test-api-key".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "firecrawl");
    }

    #[test]
    fn test_firecrawl_request_serialization() {
        let request = FirecrawlSearchRequest {
            query: "test query".to_string(),
            limit: 10,
            sources: vec!["web".to_string()],
            tbs: Some("qdr:w".to_string()),
            country: Some("US".to_string()),
            timeout: Some(30000),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"query\":\"test query\""));
        assert!(json.contains("\"limit\":10"));
        assert!(json.contains("\"tbs\":\"qdr:w\""));
    }
}
