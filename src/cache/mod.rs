//! Result caching

use crate::config::CacheConfig;
use crate::error::Result;
use crate::providers::SearchResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Cached search entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    results: Vec<SearchResult>,
    provider: String,
    #[serde(skip)]
    created_at: Option<Instant>,
    ttl_seconds: u64,
}

/// In-memory cache for search results
pub struct SearchCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    config: CacheConfig,
}

impl SearchCache {
    /// Create a new cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Generate a cache key from query and options
    fn cache_key(query: &str, provider: Option<&str>) -> String {
        match provider {
            Some(p) => format!("{}:{}", p, query.to_lowercase()),
            None => query.to_lowercase(),
        }
    }

    /// Get cached results if available and not expired
    pub fn get(&self, query: &str, provider: Option<&str>) -> Option<(Vec<SearchResult>, String)> {
        if !self.config.enabled {
            return None;
        }

        let key = Self::cache_key(query, provider);
        let entries = self.entries.read().ok()?;

        if let Some(entry) = entries.get(&key) {
            // Check if entry has expired
            if let Some(created_at) = entry.created_at {
                if created_at.elapsed() < Duration::from_secs(entry.ttl_seconds) {
                    return Some((entry.results.clone(), entry.provider.clone()));
                }
            }
        }

        None
    }

    /// Store results in cache
    pub fn set(&self, query: &str, provider: &str, results: Vec<SearchResult>) {
        if !self.config.enabled {
            return;
        }

        let key = Self::cache_key(query, Some(provider));

        if let Ok(mut entries) = self.entries.write() {
            // Evict old entries if at capacity
            if entries.len() >= self.config.max_entries {
                self.evict_oldest(&mut entries);
            }

            entries.insert(
                key,
                CacheEntry {
                    results,
                    provider: provider.to_string(),
                    created_at: Some(Instant::now()),
                    ttl_seconds: self.config.ttl_seconds,
                },
            );
        }
    }

    /// Evict the oldest entries to make room
    fn evict_oldest(&self, entries: &mut HashMap<String, CacheEntry>) {
        // Simple eviction: remove entries that are past their TTL
        entries.retain(|_, entry| {
            entry
                .created_at
                .map(|created| created.elapsed() < Duration::from_secs(entry.ttl_seconds))
                .unwrap_or(false)
        });

        // If still at capacity, remove some arbitrary entries
        while entries.len() >= self.config.max_entries {
            if let Some(key) = entries.keys().next().cloned() {
                entries.remove(&key);
            } else {
                break;
            }
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) -> Result<()> {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().map(|e| e.len()).unwrap_or(0);
        CacheStats {
            entries,
            max_entries: self.config.max_entries,
            ttl_seconds: self.config.ttl_seconds,
            enabled: self.config.enabled,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub enabled: bool,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cache Statistics:")?;
        writeln!(f, "  Enabled: {}", self.enabled)?;
        writeln!(f, "  Entries: {} / {}", self.entries, self.max_entries)?;
        writeln!(f, "  TTL: {} seconds", self.ttl_seconds)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CacheConfig {
        CacheConfig {
            enabled: true,
            ttl_seconds: 3600,
            max_entries: 100,
        }
    }

    fn create_test_result(title: &str) -> SearchResult {
        SearchResult {
            title: title.to_string(),
            url: format!("https://example.com/{}", title),
            snippet: format!("Snippet for {}", title),
            position: 1,
            published_date: None,
            source: None,
        }
    }

    #[test]
    fn test_cache_set_get() {
        let cache = SearchCache::new(test_config());

        let results = vec![create_test_result("Test")];

        cache.set("test query", "brave", results.clone());

        let cached = cache.get("test query", Some("brave"));
        assert!(cached.is_some());

        let (cached_results, provider) = cached.unwrap();
        assert_eq!(cached_results.len(), 1);
        assert_eq!(provider, "brave");
    }

    #[test]
    fn test_cache_disabled() {
        let mut config = test_config();
        config.enabled = false;
        let cache = SearchCache::new(config);

        let results = vec![create_test_result("Test")];

        cache.set("test", "brave", results);
        assert!(cache.get("test", Some("brave")).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = SearchCache::new(test_config());
        let stats = cache.stats();

        assert!(stats.enabled);
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.max_entries, 100);
    }

    #[test]
    fn test_cache_key_generation() {
        // Test with provider
        let key1 = SearchCache::cache_key("Test Query", Some("brave"));
        assert_eq!(key1, "brave:test query");

        // Test without provider
        let key2 = SearchCache::cache_key("Test Query", None);
        assert_eq!(key2, "test query");

        // Test case insensitivity
        let key3 = SearchCache::cache_key("TEST QUERY", Some("brave"));
        assert_eq!(key3, "brave:test query");
    }

    #[test]
    fn test_cache_clear() {
        let cache = SearchCache::new(test_config());
        
        cache.set("query1", "brave", vec![create_test_result("Result1")]);
        cache.set("query2", "google", vec![create_test_result("Result2")]);
        
        assert_eq!(cache.stats().entries, 2);
        
        cache.clear().unwrap();
        
        assert_eq!(cache.stats().entries, 0);
        assert!(cache.get("query1", Some("brave")).is_none());
        assert!(cache.get("query2", Some("google")).is_none());
    }

    #[test]
    fn test_cache_different_providers() {
        let cache = SearchCache::new(test_config());
        
        cache.set("same query", "brave", vec![create_test_result("Brave Result")]);
        cache.set("same query", "google", vec![create_test_result("Google Result")]);
        
        let brave_cached = cache.get("same query", Some("brave"));
        let google_cached = cache.get("same query", Some("google"));
        
        assert!(brave_cached.is_some());
        assert!(google_cached.is_some());
        
        assert_eq!(brave_cached.unwrap().0[0].title, "Brave Result");
        assert_eq!(google_cached.unwrap().0[0].title, "Google Result");
    }

    #[test]
    fn test_cache_case_insensitive_query() {
        let cache = SearchCache::new(test_config());
        
        cache.set("Test Query", "brave", vec![create_test_result("Result")]);
        
        // Should find with different case
        let cached = cache.get("test query", Some("brave"));
        assert!(cached.is_some());
        
        let cached = cache.get("TEST QUERY", Some("brave"));
        assert!(cached.is_some());
    }

    #[test]
    fn test_cache_miss_wrong_provider() {
        let cache = SearchCache::new(test_config());
        
        cache.set("query", "brave", vec![create_test_result("Result")]);
        
        // Should miss with different provider
        let cached = cache.get("query", Some("google"));
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_stats_after_operations() {
        let cache = SearchCache::new(test_config());
        
        assert_eq!(cache.stats().entries, 0);
        
        cache.set("query1", "brave", vec![create_test_result("Result1")]);
        assert_eq!(cache.stats().entries, 1);
        
        cache.set("query2", "brave", vec![create_test_result("Result2")]);
        assert_eq!(cache.stats().entries, 2);
        
        cache.clear().unwrap();
        assert_eq!(cache.stats().entries, 0);
    }

    #[test]
    fn test_cache_max_entries_eviction() {
        let mut config = test_config();
        config.max_entries = 3;
        let cache = SearchCache::new(config);
        
        cache.set("query1", "brave", vec![create_test_result("Result1")]);
        cache.set("query2", "brave", vec![create_test_result("Result2")]);
        cache.set("query3", "brave", vec![create_test_result("Result3")]);
        
        assert_eq!(cache.stats().entries, 3);
        
        // Adding a 4th entry should trigger eviction
        cache.set("query4", "brave", vec![create_test_result("Result4")]);
        
        // Should have at most max_entries
        assert!(cache.stats().entries <= 3);
    }

    #[test]
    fn test_cache_stats_display() {
        let cache = SearchCache::new(test_config());
        cache.set("query", "brave", vec![create_test_result("Result")]);
        
        let stats = cache.stats();
        let display = format!("{}", stats);
        
        assert!(display.contains("Cache Statistics:"));
        assert!(display.contains("Enabled: true"));
        assert!(display.contains("Entries: 1"));
        assert!(display.contains("TTL:"));
    }

    #[test]
    fn test_cache_multiple_results() {
        let cache = SearchCache::new(test_config());
        
        let results = vec![
            create_test_result("Result1"),
            create_test_result("Result2"),
            create_test_result("Result3"),
        ];
        
        cache.set("query", "brave", results);
        
        let cached = cache.get("query", Some("brave")).unwrap();
        assert_eq!(cached.0.len(), 3);
        assert_eq!(cached.0[0].title, "Result1");
        assert_eq!(cached.0[1].title, "Result2");
        assert_eq!(cached.0[2].title, "Result3");
    }
}
