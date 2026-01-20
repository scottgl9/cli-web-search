//! Configuration loading and saving

use super::*;
use crate::error::{Result, SearchError};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Environment variable prefix for configuration
const ENV_PREFIX: &str = "CLI_WEB_SEARCH";

/// Get the configuration directory path
pub fn config_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "cli-web-search", "cli-web-search")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| SearchError::Config("Could not determine config directory".to_string()))
}

/// Get the configuration file path
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yaml"))
}

/// Get the cache directory path
#[allow(dead_code)]
pub fn cache_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "cli-web-search", "cli-web-search")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| SearchError::Config("Could not determine cache directory".to_string()))
}

/// Load configuration from file and environment variables
pub fn load_config() -> Result<Config> {
    let mut config = load_config_file().unwrap_or_default();

    // Override with environment variables
    apply_env_overrides(&mut config);

    Ok(config)
}

/// Load configuration from file only
fn load_config_file() -> Result<Config> {
    let path = config_path()?;

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)?;
    let config: Config = serde_yaml::from_str(&content)?;

    Ok(config)
}

/// Apply environment variable overrides to config
fn apply_env_overrides(config: &mut Config) {
    // Brave API key
    if let Ok(api_key) = std::env::var(format!("{}_BRAVE_API_KEY", ENV_PREFIX)) {
        if config.providers.brave.is_none() {
            config.providers.brave = Some(BraveConfig {
                api_key: api_key.clone(),
                enabled: true,
            });
        } else if let Some(ref mut brave) = config.providers.brave {
            brave.api_key = api_key;
        }
    }

    // Google API key and CX
    let google_api_key = std::env::var(format!("{}_GOOGLE_API_KEY", ENV_PREFIX)).ok();
    let google_cx = std::env::var(format!("{}_GOOGLE_CX", ENV_PREFIX)).ok();

    if let (Some(api_key), Some(cx)) = (google_api_key.clone(), google_cx.clone()) {
        if config.providers.google.is_none() {
            config.providers.google = Some(GoogleConfig {
                api_key,
                cx,
                enabled: true,
            });
        }
    }
    if let Some(ref mut google) = config.providers.google {
        if let Some(api_key) = google_api_key {
            google.api_key = api_key;
        }
        if let Some(cx) = google_cx {
            google.cx = cx;
        }
    }

    // Tavily API key
    if let Ok(api_key) = std::env::var(format!("{}_TAVILY_API_KEY", ENV_PREFIX)) {
        if config.providers.tavily.is_none() {
            config.providers.tavily = Some(TavilyConfig {
                api_key: api_key.clone(),
                enabled: true,
            });
        } else if let Some(ref mut tavily) = config.providers.tavily {
            tavily.api_key = api_key;
        }
    }

    // DuckDuckGo enabled (no API key needed)
    if let Ok(enabled) = std::env::var(format!("{}_DUCKDUCKGO_ENABLED", ENV_PREFIX)) {
        let is_enabled = enabled.parse().unwrap_or(false);
        if config.providers.duckduckgo.is_none() {
            config.providers.duckduckgo = Some(DuckDuckGoConfig {
                enabled: is_enabled,
            });
        } else if let Some(ref mut ddg) = config.providers.duckduckgo {
            ddg.enabled = is_enabled;
        }
    }

    // Serper API key
    if let Ok(api_key) = std::env::var(format!("{}_SERPER_API_KEY", ENV_PREFIX)) {
        if config.providers.serper.is_none() {
            config.providers.serper = Some(SerperConfig {
                api_key: api_key.clone(),
                enabled: true,
            });
        } else if let Some(ref mut serper) = config.providers.serper {
            serper.api_key = api_key;
        }
    }

    // Firecrawl API key
    if let Ok(api_key) = std::env::var(format!("{}_FIRECRAWL_API_KEY", ENV_PREFIX)) {
        if config.providers.firecrawl.is_none() {
            config.providers.firecrawl = Some(FirecrawlConfig {
                api_key: api_key.clone(),
                enabled: true,
            });
        } else if let Some(ref mut firecrawl) = config.providers.firecrawl {
            firecrawl.api_key = api_key;
        }
    }

    // SerpAPI API key
    if let Ok(api_key) = std::env::var(format!("{}_SERPAPI_API_KEY", ENV_PREFIX)) {
        if config.providers.serpapi.is_none() {
            config.providers.serpapi = Some(SerpApiConfig {
                api_key: api_key.clone(),
                enabled: true,
            });
        } else if let Some(ref mut serpapi) = config.providers.serpapi {
            serpapi.api_key = api_key;
        }
    }

    // Bing API key
    if let Ok(api_key) = std::env::var(format!("{}_BING_API_KEY", ENV_PREFIX)) {
        if config.providers.bing.is_none() {
            config.providers.bing = Some(BingConfig {
                api_key: api_key.clone(),
                enabled: true,
            });
        } else if let Some(ref mut bing) = config.providers.bing {
            bing.api_key = api_key;
        }
    }

    // Default provider override
    if let Ok(provider) = std::env::var(format!("{}_DEFAULT_PROVIDER", ENV_PREFIX)) {
        config.default_provider = Some(provider);
    }
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_yaml::to_string(config)?;
    fs::write(&path, content)?;

    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

/// Initialize configuration with interactive wizard
pub fn init_config_interactive() -> Result<Config> {
    // For now, create a default config
    // TODO: Implement interactive wizard with prompts
    let config = Config::default();
    save_config(&config)?;
    Ok(config)
}

/// Set a specific configuration value by key path
pub fn set_config_value(key: &str, value: &str) -> Result<()> {
    let mut config = load_config()?;

    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["default_provider"] => {
            config.default_provider = Some(value.to_string());
        }
        ["providers", "brave", "api_key"] => {
            if config.providers.brave.is_none() {
                config.providers.brave = Some(BraveConfig {
                    api_key: value.to_string(),
                    enabled: true,
                });
            } else if let Some(ref mut brave) = config.providers.brave {
                brave.api_key = value.to_string();
            }
        }
        ["providers", "brave", "enabled"] => {
            if let Some(ref mut brave) = config.providers.brave {
                brave.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "google", "api_key"] => {
            if config.providers.google.is_none() {
                config.providers.google = Some(GoogleConfig {
                    api_key: value.to_string(),
                    cx: String::new(),
                    enabled: true,
                });
            } else if let Some(ref mut google) = config.providers.google {
                google.api_key = value.to_string();
            }
        }
        ["providers", "google", "cx"] => {
            if let Some(ref mut google) = config.providers.google {
                google.cx = value.to_string();
            } else {
                config.providers.google = Some(GoogleConfig {
                    api_key: String::new(),
                    cx: value.to_string(),
                    enabled: true,
                });
            }
        }
        ["providers", "google", "enabled"] => {
            if let Some(ref mut google) = config.providers.google {
                google.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "tavily", "api_key"] => {
            if config.providers.tavily.is_none() {
                config.providers.tavily = Some(TavilyConfig {
                    api_key: value.to_string(),
                    enabled: true,
                });
            } else if let Some(ref mut tavily) = config.providers.tavily {
                tavily.api_key = value.to_string();
            }
        }
        ["providers", "tavily", "enabled"] => {
            if let Some(ref mut tavily) = config.providers.tavily {
                tavily.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "duckduckgo", "enabled"] => {
            if config.providers.duckduckgo.is_none() {
                config.providers.duckduckgo = Some(DuckDuckGoConfig {
                    enabled: value.parse().unwrap_or(true),
                });
            } else if let Some(ref mut ddg) = config.providers.duckduckgo {
                ddg.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "serper", "api_key"] => {
            if config.providers.serper.is_none() {
                config.providers.serper = Some(SerperConfig {
                    api_key: value.to_string(),
                    enabled: true,
                });
            } else if let Some(ref mut serper) = config.providers.serper {
                serper.api_key = value.to_string();
            }
        }
        ["providers", "serper", "enabled"] => {
            if let Some(ref mut serper) = config.providers.serper {
                serper.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "firecrawl", "api_key"] => {
            if config.providers.firecrawl.is_none() {
                config.providers.firecrawl = Some(FirecrawlConfig {
                    api_key: value.to_string(),
                    enabled: true,
                });
            } else if let Some(ref mut firecrawl) = config.providers.firecrawl {
                firecrawl.api_key = value.to_string();
            }
        }
        ["providers", "firecrawl", "enabled"] => {
            if let Some(ref mut firecrawl) = config.providers.firecrawl {
                firecrawl.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "serpapi", "api_key"] => {
            if config.providers.serpapi.is_none() {
                config.providers.serpapi = Some(SerpApiConfig {
                    api_key: value.to_string(),
                    enabled: true,
                });
            } else if let Some(ref mut serpapi) = config.providers.serpapi {
                serpapi.api_key = value.to_string();
            }
        }
        ["providers", "serpapi", "enabled"] => {
            if let Some(ref mut serpapi) = config.providers.serpapi {
                serpapi.enabled = value.parse().unwrap_or(true);
            }
        }
        ["providers", "bing", "api_key"] => {
            if config.providers.bing.is_none() {
                config.providers.bing = Some(BingConfig {
                    api_key: value.to_string(),
                    enabled: true,
                });
            } else if let Some(ref mut bing) = config.providers.bing {
                bing.api_key = value.to_string();
            }
        }
        ["providers", "bing", "enabled"] => {
            if let Some(ref mut bing) = config.providers.bing {
                bing.enabled = value.parse().unwrap_or(true);
            }
        }
        ["defaults", "num_results"] => {
            config.defaults.num_results = value.parse().unwrap_or(10);
        }
        ["defaults", "safe_search"] => {
            config.defaults.safe_search = value.to_string();
        }
        ["defaults", "timeout"] => {
            config.defaults.timeout = value.parse().unwrap_or(30);
        }
        ["defaults", "format"] => {
            config.defaults.format = value.to_string();
        }
        ["cache", "enabled"] => {
            config.cache.enabled = value.parse().unwrap_or(true);
        }
        ["cache", "ttl_seconds"] => {
            config.cache.ttl_seconds = value.parse().unwrap_or(3600);
        }
        ["cache", "max_entries"] => {
            config.cache.max_entries = value.parse().unwrap_or(1000);
        }
        _ => {
            return Err(SearchError::Config(format!(
                "Unknown configuration key: {}",
                key
            )));
        }
    }

    save_config(&config)?;
    Ok(())
}

/// Get a specific configuration value by key path
pub fn get_config_value(key: &str) -> Result<Option<String>> {
    let config = load_config()?;
    let map = config.to_flat_map();
    Ok(map.get(key).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let path = config_path();
        assert!(path.is_ok());
    }

    #[test]
    fn test_default_config_loading() {
        let config = Config::default();
        assert!(config.default_provider.is_none());
        assert_eq!(config.defaults.num_results, 10);
    }
}
