//! Error types for cli-web-search

use thiserror::Error;

/// Main error type for the application
#[derive(Error, Debug)]
pub enum SearchError {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// API errors from search providers
    #[error("API error from {provider}: {message}")]
    Api { provider: String, message: String },

    /// Rate limit exceeded
    #[error("Rate limited by {provider}{}", .retry_after.map(|s| format!(", retry after {} seconds", s)).unwrap_or_default())]
    RateLimited {
        provider: String,
        retry_after: Option<u64>,
    },

    /// Invalid or missing API key
    #[error("Invalid API key for {provider}")]
    InvalidApiKey { provider: String },

    /// Missing API key
    #[error("Missing API key for {provider}. Set {env_var} or configure in ~/.config/cli-web-search/config.yaml")]
    MissingApiKey { provider: String, env_var: String },

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing errors
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON parsing errors
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing errors
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    /// Provider not found
    #[error("Unknown provider: {0}. Available providers: brave, google, duckduckgo, tavily")]
    #[allow(dead_code)]
    UnknownProvider(String),

    /// No providers configured
    #[error("No search providers configured. Run `cli-web-search config init` to set up.")]
    NoProvidersConfigured,

    /// All providers failed
    #[error("All providers failed. Last error: {0}")]
    AllProvidersFailed(String),

    /// Timeout
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),
}

impl SearchError {
    /// Create an API error for a specific provider
    pub fn api(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Api {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limited(provider: impl Into<String>, retry_after: Option<u64>) -> Self {
        Self::RateLimited {
            provider: provider.into(),
            retry_after,
        }
    }

    /// Create an invalid API key error
    pub fn invalid_api_key(provider: impl Into<String>) -> Self {
        Self::InvalidApiKey {
            provider: provider.into(),
        }
    }

    /// Create a missing API key error
    pub fn missing_api_key(provider: impl Into<String>, env_var: impl Into<String>) -> Self {
        Self::MissingApiKey {
            provider: provider.into(),
            env_var: env_var.into(),
        }
    }
}

/// Result type alias for SearchError
pub type Result<T> = std::result::Result<T, SearchError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_constructor() {
        let err = SearchError::api("brave", "Connection failed");
        match err {
            SearchError::Api { provider, message } => {
                assert_eq!(provider, "brave");
                assert_eq!(message, "Connection failed");
            }
            _ => panic!("Expected Api error"),
        }
    }

    #[test]
    fn test_rate_limited_with_retry_after() {
        let err = SearchError::rate_limited("google", Some(60));
        match err {
            SearchError::RateLimited {
                provider,
                retry_after,
            } => {
                assert_eq!(provider, "google");
                assert_eq!(retry_after, Some(60));
            }
            _ => panic!("Expected RateLimited error"),
        }
    }

    #[test]
    fn test_rate_limited_without_retry_after() {
        let err = SearchError::rate_limited("tavily", None);
        match err {
            SearchError::RateLimited {
                provider,
                retry_after,
            } => {
                assert_eq!(provider, "tavily");
                assert!(retry_after.is_none());
            }
            _ => panic!("Expected RateLimited error"),
        }
    }

    #[test]
    fn test_invalid_api_key_constructor() {
        let err = SearchError::invalid_api_key("serper");
        match err {
            SearchError::InvalidApiKey { provider } => {
                assert_eq!(provider, "serper");
            }
            _ => panic!("Expected InvalidApiKey error"),
        }
    }

    #[test]
    fn test_missing_api_key_constructor() {
        let err = SearchError::missing_api_key("brave", "CLI_WEB_SEARCH_BRAVE_API_KEY");
        match err {
            SearchError::MissingApiKey { provider, env_var } => {
                assert_eq!(provider, "brave");
                assert_eq!(env_var, "CLI_WEB_SEARCH_BRAVE_API_KEY");
            }
            _ => panic!("Expected MissingApiKey error"),
        }
    }

    #[test]
    fn test_api_error_display() {
        let err = SearchError::api("brave", "Connection timeout");
        let msg = format!("{}", err);
        assert!(msg.contains("brave"));
        assert!(msg.contains("Connection timeout"));
    }

    #[test]
    fn test_rate_limited_display_with_retry() {
        let err = SearchError::rate_limited("google", Some(30));
        let msg = format!("{}", err);
        assert!(msg.contains("google"));
        assert!(msg.contains("30 seconds"));
    }

    #[test]
    fn test_rate_limited_display_without_retry() {
        let err = SearchError::rate_limited("tavily", None);
        let msg = format!("{}", err);
        assert!(msg.contains("tavily"));
        assert!(!msg.contains("seconds"));
    }

    #[test]
    fn test_invalid_api_key_display() {
        let err = SearchError::invalid_api_key("bing");
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid API key"));
        assert!(msg.contains("bing"));
    }

    #[test]
    fn test_missing_api_key_display() {
        let err = SearchError::missing_api_key("firecrawl", "CLI_WEB_SEARCH_FIRECRAWL_API_KEY");
        let msg = format!("{}", err);
        assert!(msg.contains("firecrawl"));
        assert!(msg.contains("CLI_WEB_SEARCH_FIRECRAWL_API_KEY"));
    }

    #[test]
    fn test_config_error_display() {
        let err = SearchError::Config("Invalid configuration file".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid configuration file"));
    }

    #[test]
    fn test_no_providers_configured_display() {
        let err = SearchError::NoProvidersConfigured;
        let msg = format!("{}", err);
        assert!(msg.contains("No search providers configured"));
        assert!(msg.contains("config init"));
    }

    #[test]
    fn test_all_providers_failed_display() {
        let err = SearchError::AllProvidersFailed("Network timeout".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("All providers failed"));
        assert!(msg.contains("Network timeout"));
    }

    #[test]
    fn test_timeout_error_display() {
        let err = SearchError::Timeout(30);
        let msg = format!("{}", err);
        assert!(msg.contains("30 seconds"));
    }

    #[test]
    fn test_unknown_provider_display() {
        let err = SearchError::UnknownProvider("foobar".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("foobar"));
        assert!(msg.contains("Unknown provider"));
    }

    #[test]
    fn test_error_debug_impl() {
        let err = SearchError::api("test", "error");
        let debug = format!("{:?}", err);
        assert!(debug.contains("Api"));
        assert!(debug.contains("test"));
    }
}
