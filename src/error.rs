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
