//! Configuration management for cli-web-search

mod loader;

pub use loader::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Default provider to use
    #[serde(default)]
    pub default_provider: Option<String>,

    /// Provider configurations
    #[serde(default)]
    pub providers: ProvidersConfig,

    /// Fallback order when primary provider fails
    #[serde(default)]
    pub fallback_order: Vec<String>,

    /// Default options
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Cache settings
    #[serde(default)]
    pub cache: CacheConfig,
}

/// Provider-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvidersConfig {
    /// Brave Search configuration
    #[serde(default)]
    pub brave: Option<BraveConfig>,

    /// Google CSE configuration
    #[serde(default)]
    pub google: Option<GoogleConfig>,

    /// DuckDuckGo configuration
    #[serde(default)]
    pub duckduckgo: Option<DuckDuckGoConfig>,

    /// Tavily configuration
    #[serde(default)]
    pub tavily: Option<TavilyConfig>,

    /// Serper configuration
    #[serde(default)]
    pub serper: Option<SerperConfig>,

    /// Firecrawl configuration
    #[serde(default)]
    pub firecrawl: Option<FirecrawlConfig>,
}

/// Brave Search provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveConfig {
    /// API key for Brave Search
    pub api_key: String,

    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Google Custom Search Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    /// API key for Google CSE
    pub api_key: String,

    /// Custom Search Engine ID
    pub cx: String,

    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// DuckDuckGo configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckDuckGoConfig {
    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Tavily configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavilyConfig {
    /// API key for Tavily
    pub api_key: String,

    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Serper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerperConfig {
    /// API key for Serper
    pub api_key: String,

    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Firecrawl configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirecrawlConfig {
    /// API key for Firecrawl
    pub api_key: String,

    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Default options configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default number of results
    #[serde(default = "default_num_results")]
    pub num_results: usize,

    /// Default safe search level
    #[serde(default = "default_safe_search")]
    pub safe_search: String,

    /// Default timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Default output format
    #[serde(default = "default_format")]
    pub format: String,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            num_results: default_num_results(),
            safe_search: default_safe_search(),
            timeout: default_timeout(),
            format: default_format(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Time-to-live in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,

    /// Maximum number of cached entries
    #[serde(default = "default_cache_max_entries")]
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: default_cache_ttl(),
            max_entries: default_cache_max_entries(),
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_num_results() -> usize {
    10
}

fn default_safe_search() -> String {
    "moderate".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_format() -> String {
    "text".to_string()
}

fn default_cache_ttl() -> u64 {
    3600
}

fn default_cache_max_entries() -> usize {
    1000
}

impl Config {
    /// Get a list of enabled providers
    #[allow(dead_code)]
    pub fn enabled_providers(&self) -> Vec<String> {
        let mut providers = Vec::new();

        if let Some(ref brave) = self.providers.brave {
            if brave.enabled {
                providers.push("brave".to_string());
            }
        }
        if let Some(ref google) = self.providers.google {
            if google.enabled {
                providers.push("google".to_string());
            }
        }
        if let Some(ref ddg) = self.providers.duckduckgo {
            if ddg.enabled {
                providers.push("duckduckgo".to_string());
            }
        }
        if let Some(ref tavily) = self.providers.tavily {
            if tavily.enabled {
                providers.push("tavily".to_string());
            }
        }
        if let Some(ref serper) = self.providers.serper {
            if serper.enabled {
                providers.push("serper".to_string());
            }
        }
        if let Some(ref firecrawl) = self.providers.firecrawl {
            if firecrawl.enabled {
                providers.push("firecrawl".to_string());
            }
        }

        providers
    }

    /// Get the effective default provider
    #[allow(dead_code)]
    pub fn effective_default_provider(&self) -> Option<String> {
        // First try explicit default
        if let Some(ref default) = self.default_provider {
            if self.enabled_providers().contains(default) {
                return Some(default.clone());
            }
        }

        // Fall back to first in fallback order
        for provider in &self.fallback_order {
            if self.enabled_providers().contains(provider) {
                return Some(provider.clone());
            }
        }

        // Fall back to first enabled provider
        self.enabled_providers().into_iter().next()
    }

    /// Get configuration as a flat key-value map (for display)
    pub fn to_flat_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if let Some(ref default) = self.default_provider {
            map.insert("default_provider".to_string(), default.clone());
        }

        if let Some(ref brave) = self.providers.brave {
            map.insert(
                "providers.brave.api_key".to_string(),
                mask_api_key(&brave.api_key),
            );
            map.insert(
                "providers.brave.enabled".to_string(),
                brave.enabled.to_string(),
            );
        }

        if let Some(ref google) = self.providers.google {
            map.insert(
                "providers.google.api_key".to_string(),
                mask_api_key(&google.api_key),
            );
            map.insert("providers.google.cx".to_string(), google.cx.clone());
            map.insert(
                "providers.google.enabled".to_string(),
                google.enabled.to_string(),
            );
        }

        if let Some(ref tavily) = self.providers.tavily {
            map.insert(
                "providers.tavily.api_key".to_string(),
                mask_api_key(&tavily.api_key),
            );
            map.insert(
                "providers.tavily.enabled".to_string(),
                tavily.enabled.to_string(),
            );
        }

        if let Some(ref ddg) = self.providers.duckduckgo {
            map.insert(
                "providers.duckduckgo.enabled".to_string(),
                ddg.enabled.to_string(),
            );
        }

        if let Some(ref serper) = self.providers.serper {
            map.insert(
                "providers.serper.api_key".to_string(),
                mask_api_key(&serper.api_key),
            );
            map.insert(
                "providers.serper.enabled".to_string(),
                serper.enabled.to_string(),
            );
        }

        if let Some(ref firecrawl) = self.providers.firecrawl {
            map.insert(
                "providers.firecrawl.api_key".to_string(),
                mask_api_key(&firecrawl.api_key),
            );
            map.insert(
                "providers.firecrawl.enabled".to_string(),
                firecrawl.enabled.to_string(),
            );
        }

        map.insert(
            "defaults.num_results".to_string(),
            self.defaults.num_results.to_string(),
        );
        map.insert(
            "defaults.safe_search".to_string(),
            self.defaults.safe_search.clone(),
        );
        map.insert(
            "defaults.timeout".to_string(),
            self.defaults.timeout.to_string(),
        );
        map.insert("defaults.format".to_string(), self.defaults.format.clone());

        map.insert("cache.enabled".to_string(), self.cache.enabled.to_string());
        map.insert(
            "cache.ttl_seconds".to_string(),
            self.cache.ttl_seconds.to_string(),
        );
        map.insert(
            "cache.max_entries".to_string(),
            self.cache.max_entries.to_string(),
        );

        map
    }
}

/// Mask an API key for display (show first 4 and last 4 chars)
fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "*".repeat(key.len())
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.default_provider.is_none());
        assert_eq!(config.defaults.num_results, 10);
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("abcd1234efgh5678"), "abcd...5678");
        assert_eq!(mask_api_key("short"), "*****");
    }

    #[test]
    fn test_enabled_providers() {
        let mut config = Config::default();
        config.providers.brave = Some(BraveConfig {
            api_key: "test".to_string(),
            enabled: true,
        });
        config.providers.google = Some(GoogleConfig {
            api_key: "test".to_string(),
            cx: "cx".to_string(),
            enabled: false,
        });

        let enabled = config.enabled_providers();
        assert!(enabled.contains(&"brave".to_string()));
        assert!(!enabled.contains(&"google".to_string()));
    }

    #[test]
    fn test_defaults_config() {
        let defaults = DefaultsConfig::default();
        assert_eq!(defaults.num_results, 10);
        assert_eq!(defaults.safe_search, "moderate");
        assert_eq!(defaults.timeout, 30);
        assert_eq!(defaults.format, "text");
    }

    #[test]
    fn test_cache_config_default() {
        let cache = CacheConfig::default();
        assert!(cache.enabled);
        assert_eq!(cache.ttl_seconds, 3600);
        assert_eq!(cache.max_entries, 1000);
    }

    #[test]
    fn test_providers_config_default() {
        let providers = ProvidersConfig::default();
        assert!(providers.brave.is_none());
        assert!(providers.google.is_none());
        assert!(providers.duckduckgo.is_none());
        assert!(providers.tavily.is_none());
        assert!(providers.serper.is_none());
        assert!(providers.firecrawl.is_none());
    }

    #[test]
    fn test_mask_api_key_edge_cases() {
        // Exactly 8 characters
        assert_eq!(mask_api_key("12345678"), "********");
        // Empty string
        assert_eq!(mask_api_key(""), "");
        // Very long key
        let long_key = "a".repeat(100);
        let masked = mask_api_key(&long_key);
        assert!(masked.starts_with("aaaa"));
        assert!(masked.ends_with("aaaa"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn test_enabled_providers_all() {
        let mut config = Config::default();
        config.providers.brave = Some(BraveConfig {
            api_key: "key".to_string(),
            enabled: true,
        });
        config.providers.google = Some(GoogleConfig {
            api_key: "key".to_string(),
            cx: "cx".to_string(),
            enabled: true,
        });
        config.providers.duckduckgo = Some(DuckDuckGoConfig { enabled: true });
        config.providers.tavily = Some(TavilyConfig {
            api_key: "key".to_string(),
            enabled: true,
        });
        config.providers.serper = Some(SerperConfig {
            api_key: "key".to_string(),
            enabled: true,
        });
        config.providers.firecrawl = Some(FirecrawlConfig {
            api_key: "key".to_string(),
            enabled: true,
        });

        let enabled = config.enabled_providers();
        assert_eq!(enabled.len(), 6);
        assert!(enabled.contains(&"brave".to_string()));
        assert!(enabled.contains(&"google".to_string()));
        assert!(enabled.contains(&"duckduckgo".to_string()));
        assert!(enabled.contains(&"tavily".to_string()));
        assert!(enabled.contains(&"serper".to_string()));
        assert!(enabled.contains(&"firecrawl".to_string()));
    }

    #[test]
    fn test_to_flat_map() {
        let mut config = Config::default();
        config.providers.brave = Some(BraveConfig {
            api_key: "test-api-key-12345".to_string(),
            enabled: true,
        });

        let map = config.to_flat_map();

        // Check that API key is masked
        let brave_key = map.get("providers.brave.api_key").unwrap();
        assert!(brave_key.contains("..."));
        assert!(!brave_key.contains("test-api-key-12345"));

        // Check enabled
        assert_eq!(map.get("providers.brave.enabled").unwrap(), "true");

        // Check defaults
        assert_eq!(map.get("defaults.num_results").unwrap(), "10");
        assert_eq!(map.get("defaults.safe_search").unwrap(), "moderate");

        // Check cache
        assert_eq!(map.get("cache.enabled").unwrap(), "true");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("defaults"));
        assert!(yaml.contains("cache"));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
default_provider: brave
providers:
  brave:
    api_key: "test-key"
    enabled: true
defaults:
  num_results: 5
  safe_search: strict
  timeout: 60
  format: json
cache:
  enabled: false
  ttl_seconds: 1800
  max_entries: 500
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.default_provider, Some("brave".to_string()));
        assert!(config.providers.brave.is_some());
        assert_eq!(config.providers.brave.as_ref().unwrap().api_key, "test-key");
        assert_eq!(config.defaults.num_results, 5);
        assert_eq!(config.defaults.safe_search, "strict");
        assert!(!config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 1800);
    }

    #[test]
    fn test_fallback_order() {
        let config = Config {
            fallback_order: vec![
                "brave".to_string(),
                "google".to_string(),
                "tavily".to_string(),
            ],
            ..Default::default()
        };

        assert_eq!(config.fallback_order.len(), 3);
        assert_eq!(config.fallback_order[0], "brave");
    }
}
