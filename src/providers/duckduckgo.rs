//! DuckDuckGo Instant Answer API provider
//!
//! Note: DuckDuckGo's Instant Answer API provides instant answers and related topics,
//! not traditional web search results. It's free and doesn't require an API key.
//! For full web search results, DuckDuckGo doesn't provide a public API.

use super::{SearchOptions, SearchProvider, SearchResult};
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const DDG_API_URL: &str = "https://api.duckduckgo.com/";

/// DuckDuckGo Instant Answer API provider
pub struct DuckDuckGoProvider {
    client: Client,
    enabled: bool,
}

impl DuckDuckGoProvider {
    /// Create a new DuckDuckGo provider
    pub fn new(enabled: bool) -> Self {
        Self {
            client: Client::new(),
            enabled,
        }
    }
}

#[async_trait]
impl SearchProvider for DuckDuckGoProvider {
    fn name(&self) -> &'static str {
        "duckduckgo"
    }

    fn is_configured(&self) -> bool {
        // DuckDuckGo doesn't require API key, just needs to be enabled
        self.enabled
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        if !self.is_configured() {
            return Err(SearchError::Api {
                provider: "duckduckgo".to_string(),
                message: "DuckDuckGo provider is not enabled".to_string(),
            });
        }

        let response = self
            .client
            .get(DDG_API_URL)
            .query(&[
                ("q", query),
                ("format", "json"),
                ("no_html", "1"),
                ("skip_disambig", "1"),
            ])
            .timeout(options.timeout)
            .send()
            .await?;

        let status = response.status();
        if status == 429 {
            return Err(SearchError::rate_limited("duckduckgo", None));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::api(
                "duckduckgo",
                format!("HTTP {}: {}", status, error_text),
            ));
        }

        let ddg_response: DdgResponse = response.json().await?;

        let mut results = Vec::new();
        let mut position = 1;

        // Add the abstract (main answer) if present
        if !ddg_response.abstract_text.is_empty() && !ddg_response.abstract_url.is_empty() {
            results.push(SearchResult {
                title: ddg_response.heading.clone(),
                url: ddg_response.abstract_url.clone(),
                snippet: ddg_response.abstract_text.clone(),
                position,
                published_date: None,
                source: ddg_response.abstract_source.clone(),
            });
            position += 1;
        }

        // Add related topics
        for topic in ddg_response.related_topics {
            if position > options.num_results {
                break;
            }

            match topic {
                DdgTopic::Result(result) => {
                    if !result.first_url.is_empty() {
                        results.push(SearchResult {
                            title: result.text.chars().take(100).collect::<String>()
                                + if result.text.len() > 100 { "..." } else { "" },
                            url: result.first_url,
                            snippet: result.text,
                            position,
                            published_date: None,
                            source: None,
                        });
                        position += 1;
                    }
                }
                DdgTopic::Category(category) => {
                    // Process topics within the category
                    for sub_topic in category.topics {
                        if position > options.num_results {
                            break;
                        }
                        if !sub_topic.first_url.is_empty() {
                            results.push(SearchResult {
                                title: sub_topic.text.chars().take(100).collect::<String>()
                                    + if sub_topic.text.len() > 100 { "..." } else { "" },
                                url: sub_topic.first_url,
                                snippet: sub_topic.text,
                                position,
                                published_date: None,
                                source: None,
                            });
                            position += 1;
                        }
                    }
                }
            }
        }

        // Add results from Results array if present
        for result in ddg_response.results {
            if position > options.num_results {
                break;
            }

            if !result.first_url.is_empty() {
                results.push(SearchResult {
                    title: result.text.chars().take(100).collect::<String>()
                        + if result.text.len() > 100 { "..." } else { "" },
                    url: result.first_url,
                    snippet: result.text,
                    position,
                    published_date: None,
                    source: None,
                });
                position += 1;
            }
        }

        // Truncate to requested number of results
        results.truncate(options.num_results);

        Ok(results)
    }

    async fn validate_api_key(&self) -> Result<bool> {
        // DuckDuckGo doesn't require an API key
        // Just verify we can reach the API
        if !self.is_configured() {
            return Ok(false);
        }

        let response = self
            .client
            .get(DDG_API_URL)
            .query(&[("q", "test"), ("format", "json")])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

// DuckDuckGo API response structures

#[derive(Debug, Deserialize)]
struct DdgResponse {
    /// Main heading
    #[serde(default, rename = "Heading")]
    heading: String,

    /// Abstract text (main answer)
    #[serde(default, rename = "AbstractText")]
    abstract_text: String,

    /// Abstract URL (source)
    #[serde(default, rename = "AbstractURL")]
    abstract_url: String,

    /// Abstract source name
    #[serde(default, rename = "AbstractSource")]
    abstract_source: Option<String>,

    /// Related topics
    #[serde(default, rename = "RelatedTopics")]
    related_topics: Vec<DdgTopic>,

    /// Direct results
    #[serde(default, rename = "Results")]
    results: Vec<DdgResult>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DdgTopic {
    Result(DdgResult),
    Category(DdgCategory),
}

#[derive(Debug, Deserialize)]
struct DdgResult {
    /// Result text
    #[serde(default, rename = "Text")]
    text: String,

    /// First URL
    #[serde(default, rename = "FirstURL")]
    first_url: String,
}

#[derive(Debug, Deserialize)]
struct DdgCategory {
    /// Category name
    #[serde(default, rename = "Name")]
    name: String,

    /// Topics in this category
    #[serde(default, rename = "Topics")]
    topics: Vec<DdgResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duckduckgo_provider_not_configured() {
        let provider = DuckDuckGoProvider::new(false);
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_duckduckgo_provider_configured() {
        let provider = DuckDuckGoProvider::new(true);
        assert!(provider.is_configured());
        assert_eq!(provider.name(), "duckduckgo");
    }

    #[test]
    fn test_duckduckgo_response_parsing() {
        let json = r#"{
            "Heading": "Test",
            "AbstractText": "This is a test",
            "AbstractURL": "https://example.com",
            "AbstractSource": "Example",
            "RelatedTopics": [],
            "Results": []
        }"#;

        let response: DdgResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.heading, "Test");
        assert_eq!(response.abstract_text, "This is a test");
    }
}
