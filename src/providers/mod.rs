//! Search provider infrastructure

mod brave;
mod duckduckgo;
mod firecrawl;
mod google;
mod serper;
mod tavily;

pub use brave::BraveProvider;
pub use duckduckgo::DuckDuckGoProvider;
pub use firecrawl::FirecrawlProvider;
pub use google::GoogleProvider;
pub use serper::SerperProvider;
pub use tavily::TavilyProvider;

use crate::cli::{DateRange, SafeSearch};
use crate::error::{Result, SearchError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// Maximum number of retries per provider
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (in milliseconds)
const BASE_DELAY_MS: u64 = 500;

/// A single search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Title of the result
    pub title: String,

    /// URL of the result
    pub url: String,

    /// Snippet/description of the result
    pub snippet: String,

    /// Position in search results (1-indexed)
    pub position: usize,

    /// Optional published date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,

    /// Optional source/domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Search options passed to providers
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub num_results: usize,

    /// Safe search level
    pub safe_search: SafeSearch,

    /// Date range filter
    pub date_range: Option<DateRange>,

    /// Include only results from these domains
    pub include_domains: Option<Vec<String>>,

    /// Exclude results from these domains
    pub exclude_domains: Option<Vec<String>>,

    /// Request timeout
    pub timeout: Duration,
}

impl SearchOptions {
    pub fn new() -> Self {
        Self {
            num_results: 10,
            safe_search: SafeSearch::Moderate,
            date_range: None,
            include_domains: None,
            exclude_domains: None,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_num_results(mut self, n: usize) -> Self {
        self.num_results = n;
        self
    }

    pub fn with_safe_search(mut self, level: SafeSearch) -> Self {
        self.safe_search = level;
        self
    }

    pub fn with_date_range(mut self, range: Option<DateRange>) -> Self {
        self.date_range = range;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Trait that all search providers must implement
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &'static str;

    /// Execute a search query
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>>;

    /// Validate that the API key is working
    async fn validate_api_key(&self) -> Result<bool>;

    /// Check if the provider is configured (has API key)
    fn is_configured(&self) -> bool;
}

/// Provider registry for managing multiple providers
pub struct ProviderRegistry {
    providers: Vec<Box<dyn SearchProvider>>,
    fallback_order: Vec<String>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            fallback_order: Vec::new(),
        }
    }

    /// Add a provider to the registry
    pub fn register(&mut self, provider: Box<dyn SearchProvider>) {
        self.providers.push(provider);
    }

    /// Set the fallback order
    pub fn set_fallback_order(&mut self, order: Vec<String>) {
        self.fallback_order = order;
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<&dyn SearchProvider> {
        self.providers
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// Get all configured providers
    pub fn configured_providers(&self) -> Vec<&dyn SearchProvider> {
        self.providers
            .iter()
            .filter(|p| p.is_configured())
            .map(|p| p.as_ref())
            .collect()
    }

    /// Get providers in fallback order
    pub fn providers_in_order(&self) -> Vec<&dyn SearchProvider> {
        let mut result = Vec::new();

        // First add providers in fallback order
        for name in &self.fallback_order {
            if let Some(provider) = self.get(name) {
                if provider.is_configured() {
                    result.push(provider);
                }
            }
        }

        // Then add any remaining configured providers
        for provider in &self.providers {
            if provider.is_configured() && !result.iter().any(|p| p.name() == provider.name()) {
                result.push(provider.as_ref());
            }
        }

        result
    }

    /// Execute search with fallback and retry logic
    pub async fn search_with_fallback(
        &self,
        query: &str,
        options: &SearchOptions,
        preferred_provider: Option<&str>,
    ) -> Result<(Vec<SearchResult>, &str)> {
        let mut providers = self.providers_in_order();

        // If a preferred provider is specified, try it first
        if let Some(preferred) = preferred_provider {
            if let Some(pos) = providers.iter().position(|p| p.name() == preferred) {
                let provider = providers.remove(pos);
                providers.insert(0, provider);
            }
        }

        if providers.is_empty() {
            return Err(SearchError::NoProvidersConfigured);
        }

        let mut last_error = String::new();

        for provider in providers {
            // Try each provider with retries
            match self.search_with_retry(provider, query, options).await {
                Ok(results) => return Ok((results, provider.name())),
                Err(e) => {
                    last_error = e.to_string();
                    tracing::warn!("Provider {} failed: {}", provider.name(), e);
                    // Continue to next provider on rate limit or API errors
                    match &e {
                        SearchError::RateLimited { .. }
                        | SearchError::Api { .. }
                        | SearchError::Network(_) => continue,
                        // For other errors, stop trying
                        _ => return Err(e),
                    }
                }
            }
        }

        Err(SearchError::AllProvidersFailed(last_error))
    }

    /// Execute search with exponential backoff retry
    async fn search_with_retry(
        &self,
        provider: &dyn SearchProvider,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match provider.search(query, options).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    // Only retry on transient errors
                    let should_retry = matches!(
                        &e,
                        SearchError::Network(_) | SearchError::RateLimited { .. }
                    );

                    if !should_retry || attempt == MAX_RETRIES - 1 {
                        return Err(e);
                    }

                    // Calculate backoff delay with exponential increase
                    let delay_ms = BASE_DELAY_MS * 2u64.pow(attempt);

                    // Check if we got a Retry-After header for rate limiting
                    let delay = if let SearchError::RateLimited {
                        retry_after: Some(secs),
                        ..
                    } = &e
                    {
                        Duration::from_secs(*secs)
                    } else {
                        Duration::from_millis(delay_ms)
                    };

                    tracing::debug!(
                        "Provider {} attempt {} failed, retrying in {:?}: {}",
                        provider.name(),
                        attempt + 1,
                        delay,
                        e
                    );

                    last_error = Some(e);
                    sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| SearchError::Api {
            provider: provider.name().to_string(),
            message: "Unknown error after retries".to_string(),
        }))
    }

    /// List all providers with their status
    pub fn list_providers(&self) -> Vec<ProviderStatus> {
        self.providers
            .iter()
            .map(|p| ProviderStatus {
                name: p.name().to_string(),
                configured: p.is_configured(),
            })
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Status information for a provider
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub name: String,
    pub configured: bool,
}

/// Build a provider registry from configuration
pub fn build_registry(config: &crate::config::Config) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();

    // Register Brave provider if configured
    if let Some(ref brave_config) = config.providers.brave {
        if brave_config.enabled {
            registry.register(Box::new(BraveProvider::new(brave_config.api_key.clone())));
        }
    }

    // Register Google provider if configured
    if let Some(ref google_config) = config.providers.google {
        if google_config.enabled {
            registry.register(Box::new(GoogleProvider::new(
                google_config.api_key.clone(),
                google_config.cx.clone(),
            )));
        }
    }

    // Register DuckDuckGo provider if configured
    if let Some(ref ddg_config) = config.providers.duckduckgo {
        if ddg_config.enabled {
            registry.register(Box::new(DuckDuckGoProvider::new(true)));
        }
    }

    // Register Tavily provider if configured
    if let Some(ref tavily_config) = config.providers.tavily {
        if tavily_config.enabled {
            registry.register(Box::new(TavilyProvider::new(tavily_config.api_key.clone())));
        }
    }

    // Register Serper provider if configured
    if let Some(ref serper_config) = config.providers.serper {
        if serper_config.enabled {
            registry.register(Box::new(SerperProvider::new(serper_config.api_key.clone())));
        }
    }

    // Register Firecrawl provider if configured
    if let Some(ref firecrawl_config) = config.providers.firecrawl {
        if firecrawl_config.enabled {
            registry.register(Box::new(FirecrawlProvider::new(
                firecrawl_config.api_key.clone(),
            )));
        }
    }

    // Set fallback order
    registry.set_fallback_order(config.fallback_order.clone());

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "A test result".to_string(),
            position: 1,
            published_date: None,
            source: Some("example.com".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions::new()
            .with_num_results(5)
            .with_safe_search(SafeSearch::Strict);

        assert_eq!(options.num_results, 5);
        assert_eq!(options.safe_search, SafeSearch::Strict);
    }

    #[test]
    fn test_search_options_defaults() {
        let options = SearchOptions::new();
        assert_eq!(options.num_results, 10);
        assert_eq!(options.safe_search, SafeSearch::Moderate);
        assert!(options.date_range.is_none());
        assert!(options.include_domains.is_none());
        assert!(options.exclude_domains.is_none());
        assert_eq!(options.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_search_options_with_date_range() {
        let options = SearchOptions::new().with_date_range(Some(DateRange::Week));
        assert_eq!(options.date_range, Some(DateRange::Week));
    }

    #[test]
    fn test_search_options_with_timeout() {
        let options = SearchOptions::new().with_timeout(Duration::from_secs(60));
        assert_eq!(options.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_provider_registry_new() {
        let registry = ProviderRegistry::new();
        assert!(registry.providers.is_empty());
        assert!(registry.fallback_order.is_empty());
    }

    #[test]
    fn test_provider_registry_register() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(BraveProvider::new("test-key".to_string())));
        assert_eq!(registry.providers.len(), 1);
    }

    #[test]
    fn test_provider_registry_get() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(BraveProvider::new("test-key".to_string())));

        let provider = registry.get("brave");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "brave");

        let missing = registry.get("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_provider_registry_configured_providers() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(BraveProvider::new("test-key".to_string())));
        registry.register(Box::new(BraveProvider::new(String::new()))); // Not configured

        let configured = registry.configured_providers();
        assert_eq!(configured.len(), 1);
    }

    #[test]
    fn test_provider_registry_list_providers() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(BraveProvider::new("test-key".to_string())));
        registry.register(Box::new(TavilyProvider::new(String::new())));

        let list = registry.list_providers();
        assert_eq!(list.len(), 2);

        let brave_status = list.iter().find(|s| s.name == "brave").unwrap();
        assert!(brave_status.configured);

        let tavily_status = list.iter().find(|s| s.name == "tavily").unwrap();
        assert!(!tavily_status.configured);
    }

    #[test]
    fn test_provider_registry_fallback_order() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(BraveProvider::new("brave-key".to_string())));
        registry.register(Box::new(TavilyProvider::new("tavily-key".to_string())));
        registry.register(Box::new(GoogleProvider::new(
            "google-key".to_string(),
            "cx".to_string(),
        )));

        // Set fallback order to put tavily first
        registry.set_fallback_order(vec![
            "tavily".to_string(),
            "brave".to_string(),
            "google".to_string(),
        ]);

        let providers = registry.providers_in_order();
        assert_eq!(providers.len(), 3);
        assert_eq!(providers[0].name(), "tavily");
        assert_eq!(providers[1].name(), "brave");
        assert_eq!(providers[2].name(), "google");
    }

    #[test]
    fn test_provider_registry_providers_in_order_with_unconfigured() {
        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(BraveProvider::new("brave-key".to_string())));
        registry.register(Box::new(TavilyProvider::new(String::new()))); // Not configured

        registry.set_fallback_order(vec!["tavily".to_string(), "brave".to_string()]);

        let providers = registry.providers_in_order();
        // Only brave should be returned since tavily is not configured
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name(), "brave");
    }

    #[test]
    fn test_search_result_deserialization() {
        let json = r#"{
            "title": "Test Title",
            "url": "https://example.com",
            "snippet": "Test snippet",
            "position": 1
        }"#;

        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, "Test snippet");
        assert_eq!(result.position, 1);
        assert!(result.published_date.is_none());
        assert!(result.source.is_none());
    }

    #[test]
    fn test_search_result_with_optional_fields() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Snippet".to_string(),
            position: 1,
            published_date: Some("2024-01-01".to_string()),
            source: Some("example.com".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("published_date"));
        assert!(json.contains("2024-01-01"));
        assert!(json.contains("source"));
    }

    #[test]
    fn test_provider_status_debug() {
        let status = ProviderStatus {
            name: "brave".to_string(),
            configured: true,
        };
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("brave"));
        assert!(debug_str.contains("true"));
    }
}
