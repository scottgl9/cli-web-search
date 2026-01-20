//! Google Custom Search Engine (CSE) provider

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::cli::SafeSearch;
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const GOOGLE_CSE_API_URL: &str = "https://www.googleapis.com/customsearch/v1";

/// Google Custom Search Engine provider
pub struct GoogleProvider {
    api_key: String,
    cx: String,
    client: Client,
}

impl GoogleProvider {
    /// Create a new Google CSE provider with the given API key and CX
    pub fn new(api_key: String, cx: String) -> Self {
        Self {
            api_key,
            cx,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for GoogleProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.cx.is_empty()
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if self.api_key.is_empty() {
            return Err(SearchError::missing_api_key(
                "google",
                "CLI_WEB_SEARCH_GOOGLE_API_KEY",
            ));
        }
        if self.cx.is_empty() {
            return Err(SearchError::missing_api_key(
                "google",
                "CLI_WEB_SEARCH_GOOGLE_CX",
            ));
        }

        let safe = match options.safe_search {
            SafeSearch::Off => "off",
            SafeSearch::Moderate => "medium",
            SafeSearch::Strict => "high",
        };

        // Google CSE has a max of 10 results per request
        let num = options.num_results.min(10);

        let mut request = self
            .client
            .get(GOOGLE_CSE_API_URL)
            .query(&[
                ("key", self.api_key.as_str()),
                ("cx", self.cx.as_str()),
                ("q", query),
                ("num", &num.to_string()),
                ("safe", safe),
            ])
            .timeout(options.timeout);

        // Add date restrict if specified
        if let Some(ref date_range) = options.date_range {
            let date_restrict = match date_range {
                crate::cli::DateRange::Day => "d1",
                crate::cli::DateRange::Week => "w1",
                crate::cli::DateRange::Month => "m1",
                crate::cli::DateRange::Year => "y1",
            };
            request = request.query(&[("dateRestrict", date_restrict)]);
        }

        // Add site restrict for domain filtering
        if let Some(ref include_domains) = options.include_domains {
            if !include_domains.is_empty() {
                let site_search = include_domains.join(" OR site:");
                let site_query = format!("site:{}", site_search);
                request = request.query(&[("siteSearch", &site_query)]);
            }
        }

        let response = request.send().await?;

        let status = response.status();
        if status == 429 {
            return Err(SearchError::rate_limited("google", None));
        }

        if status == 401 || status == 403 {
            return Err(SearchError::invalid_api_key("google"));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "google",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let google_response: GoogleSearchResponse = response.json().await?;

        let results = google_response
            .items
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, item)| SearchResult {
                title: item.title,
                url: item.link,
                snippet: item.snippet.unwrap_or_default(),
                position: i + 1,
                published_date: None,
                source: item.display_link,
            })
            .collect();

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        if !self.is_configured() {
            return Ok(false);
        }

        // Do a minimal search to validate
        let response = self
            .client
            .get(GOOGLE_CSE_API_URL)
            .query(&[
                ("key", self.api_key.as_str()),
                ("cx", self.cx.as_str()),
                ("q", "test"),
                ("num", "1"),
            ])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

// Google CSE API response structures

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
    #[serde(rename = "searchInformation")]
    search_information: Option<GoogleSearchInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
    #[serde(rename = "displayLink")]
    display_link: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GoogleSearchInfo {
    #[serde(rename = "totalResults")]
    total_results: Option<String>,
    #[serde(rename = "searchTime")]
    search_time: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_provider_not_configured() {
        let provider = GoogleProvider::new(String::new(), String::new());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_google_provider_partially_configured() {
        let provider = GoogleProvider::new("api-key".to_string(), String::new());
        assert!(!provider.is_configured());

        let provider = GoogleProvider::new(String::new(), "cx".to_string());
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_google_provider_configured() {
        let provider = GoogleProvider::new("api-key".to_string(), "cx-id".to_string());
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "google");
    }
}
