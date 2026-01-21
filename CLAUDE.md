# Claude Code Development Workflow

This document provides Claude-specific instructions for developing the cli-web-search project. It extends the general guidelines in AGENTS.md with Claude Code-specific workflows.

---

## Quick Reference

```bash
# Essential commands
cargo build                    # Build the project
cargo test                     # Run tests
cargo clippy                   # Lint code
cargo fmt                      # Format code
cargo run -- "search query"    # Run with arguments
```

---

## Session Start Checklist

When beginning a development session:

1. **Read project state**:
   ```
   Read TODO.md and PROGRESS.md
   ```

2. **Check current code state**:
   ```bash
   cargo check
   cargo test
   ```

3. **Identify next task** from TODO.md

4. **Update task status**:
   - Mark the task as "in progress" in TODO.md

---

## Development Workflow

### Starting a New Feature

1. **Understand the requirement**
2. **Create a plan** before coding
3. **Implement incrementally** with tests
4. **Update documentation** as you go

### Code Changes Workflow

```
1. Make code changes
2. Run: cargo check (quick syntax/type check)
3. Run: cargo test (ensure tests pass)
4. Run: cargo clippy (lint for issues)
5. Run: cargo fmt (format code)
6. Update TODO.md and PROGRESS.md
```

### Completing a Task

1. Ensure all tests pass
2. Mark task complete in TODO.md with [x]
3. Update PROGRESS.md percentages
4. Commit with conventional commit message

---

## File Editing Priorities

When making changes, prefer editing in this order:

1. **Existing source files** - Modify rather than create
2. **Test files** - Add tests for new functionality
3. **Documentation** - Keep docs in sync with code
4. **Configuration** - Cargo.toml, config files

### Files to Update Frequently

| File | When to Update |
|------|----------------|
| `TODO.md` | Starting/completing any task |
| `PROGRESS.md` | After completing tasks |
| `Cargo.toml` | Adding dependencies |
| Source files | During development |
| Test files | With each new feature |

---

## Provider Implementation Template

When implementing a new search provider:

```rust
// src/providers/{provider_name}.rs

use async_trait::async_trait;
use crate::error::SearchError;
use crate::providers::{SearchProvider, SearchResult, SearchOptions};

pub struct {ProviderName}Provider {
    api_key: String,
    client: reqwest::Client,
}

impl {ProviderName}Provider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl SearchProvider for {ProviderName}Provider {
    fn name(&self) -> &'static str {
        "{provider_name}"
    }

    async fn search(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>, SearchError> {
        // Implementation here
        todo!()
    }

    async fn validate_api_key(&self) -> Result<bool, SearchError> {
        // Validate the API key works
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_returns_results() {
        // Test with mock or skip if no API key
    }
}
```

---

## Error Handling Patterns

### Defining Errors

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error from {provider}: {message}")]
    Api { provider: String, message: String },

    #[error("Rate limited by {provider}, retry after {retry_after:?} seconds")]
    RateLimited { provider: String, retry_after: Option<u64> },

    #[error("Invalid API key for {provider}")]
    InvalidApiKey { provider: String },

    #[error("Configuration error: {0}")]
    Config(String),
}
```

### Using Errors

```rust
// Propagate with context
let response = self.client
    .get(&url)
    .send()
    .await
    .map_err(SearchError::from)?;

// Create specific errors
if response.status() == 429 {
    return Err(SearchError::RateLimited {
        provider: self.name().to_string(),
        retry_after: parse_retry_after(&response),
    });
}
```

---

## Testing Patterns

### Mock Provider for Testing

```rust
pub struct MockProvider {
    results: Vec<SearchResult>,
}

impl MockProvider {
    pub fn with_results(results: Vec<SearchResult>) -> Self {
        Self { results }
    }
}

#[async_trait]
impl SearchProvider for MockProvider {
    async fn search(&self, _query: &str, _options: &SearchOptions) -> Result<Vec<SearchResult>, SearchError> {
        Ok(self.results.clone())
    }
}
```

### Integration Test with Real API

```rust
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_brave_real_api() {
    let api_key = std::env::var("CLI_WEB_SEARCH_BRAVE_API_KEY")
        .expect("BRAVE_API_KEY must be set");
    
    let provider = BraveProvider::new(api_key);
    let results = provider.search("rust programming", &SearchOptions::default()).await;
    
    assert!(results.is_ok());
    assert!(!results.unwrap().is_empty());
}
```

---

## Common Patterns

### Configuration Loading

```rust
pub fn load_config() -> Result<Config, SearchError> {
    // 1. Try environment variables first
    // 2. Fall back to config file
    // 3. Use defaults for non-required values
}
```

### Provider Selection

```rust
pub fn get_provider(name: &str, config: &Config) -> Result<Box<dyn SearchProvider>, SearchError> {
    match name {
        "brave" => Ok(Box::new(BraveProvider::new(config.brave_api_key()?))),
        "google" => Ok(Box::new(GoogleProvider::new(config.google_api_key()?, config.google_cx()?))),
        _ => Err(SearchError::Config(format!("Unknown provider: {}", name))),
    }
}
```

---

## Response Format Templates

### JSON Response

```rust
#[derive(Serialize)]
struct JsonResponse {
    query: String,
    provider: String,
    timestamp: String,
    results: Vec<SearchResult>,
    total_results: usize,
    search_time_ms: u64,
}
```

### Markdown Response

```rust
fn format_markdown(query: &str, results: &[SearchResult], metadata: &Metadata) -> String {
    let mut output = format!("# Search Results: {}\n\n", query);
    output.push_str(&format!("*Provider: {} | Results: {} | Time: {}ms*\n\n---\n\n",
        metadata.provider, results.len(), metadata.search_time_ms));
    
    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!("## {}. {}\n", i + 1, result.title));
        output.push_str(&format!("**URL:** {}\n\n", result.url));
        output.push_str(&format!("{}\n\n---\n\n", result.snippet));
    }
    output
}
```

---

## Debugging Tips

### Enable Detailed Logging

```rust
// In main.rs or lib.rs
use tracing_subscriber;

fn setup_logging(verbosity: u8) {
    let level = match verbosity {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
}
```

### Debugging HTTP Requests

```rust
// Add to reqwest client builder
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

// Log request details
tracing::debug!("Requesting: {} with params: {:?}", url, params);
```

---

## Session End Checklist

Before ending a development session:

1. **Ensure clean state**:
   ```bash
   cargo check && cargo test && cargo clippy
   ```

2. **Update TODO.md**: Mark completed/in-progress tasks

3. **Update PROGRESS.md**: Update percentages

4. **Summarize changes**: Note what was done and what's next

5. **Commit if appropriate**: Use conventional commits

---

## Helpful Commands

```bash
# Check what changes would be made by cargo fmt
cargo fmt -- --check

# Run clippy with all warnings as errors
cargo clippy -- -D warnings

# Generate and view documentation
cargo doc --open

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Build for release
cargo build --release

# Run with specific log level
RUST_LOG=debug cargo run -- "query"
```

---

## Project-Specific Notes

### API Rate Limits to Remember

| Provider | Free Tier Limit |
|----------|-----------------|
| Brave | 2,000/month |
| Google CSE | 100/day |
| Tavily | 1,000/month |
| Serper | 2,500 total |

### Key Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
thiserror = "1"
async-trait = "0.1"
directories = "5"
tracing = "0.1"
tracing-subscriber = "0.3"
```
