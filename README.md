# cli-web-search

A cross-platform command-line web search tool designed for AI agents and CLI power users. Search the web from your terminal with support for multiple search providers, flexible output formats, and intelligent fallback.

## Features

- **Multiple Search Providers**: Brave, Google CSE, DuckDuckGo, Tavily, Serper, and Firecrawl
- **Flexible Output**: JSON, Markdown, or plain text formats
- **Provider Fallback**: Automatic failover with retry and exponential backoff
- **Result Caching**: In-memory cache with configurable TTL
- **Search Filtering**: Date range, domain inclusion/exclusion, safe search
- **Easy Configuration**: YAML config file with environment variable overrides

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/scottgl9/cli-web-search.git
cd cli-web-search

# Build release binary
cargo build --release

# Install to PATH (optional)
cp target/release/cli-web-search ~/.local/bin/
```

### Requirements

- Rust 1.70+ (for building from source)
- At least one search provider API key (except DuckDuckGo which is free)

## Quick Start

1. **Configure a provider**:
   ```bash
   # Using environment variable
   export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"
   
   # Or using config command
   cli-web-search config set providers.brave.api_key "your-api-key"
   ```

2. **Search**:
   ```bash
   cli-web-search "rust programming tutorial"
   ```

3. **Get JSON output**:
   ```bash
   cli-web-search -f json "latest tech news"
   ```

## Usage

```
cli-web-search [OPTIONS] <QUERY>
cli-web-search <COMMAND>
```

### Basic Examples

```bash
# Simple search
cli-web-search "how to learn rust"

# Specify provider
cli-web-search -p brave "rust async programming"

# JSON output for scripting
cli-web-search -f json "weather forecast" | jq '.results[0].url'

# Markdown output
cli-web-search -f markdown "rust error handling"

# Limit results
cli-web-search -n 5 "best rust crates"

# Save to file
cli-web-search -o results.json -f json "rust web frameworks"
```

### Search Options

| Option | Short | Description |
|--------|-------|-------------|
| `--provider` | `-p` | Search provider (brave, google, ddg, tavily, serper, firecrawl) |
| `--format` | `-f` | Output format (text, json, markdown) |
| `--num-results` | `-n` | Number of results (default: 10) |
| `--output` | `-o` | Write output to file |
| `--date-range` | | Filter by date (day, week, month, year) |
| `--include-domains` | | Only include results from these domains |
| `--exclude-domains` | | Exclude results from these domains |
| `--safe-search` | | Safe search level (off, moderate, strict) |
| `--no-cache` | | Bypass result cache |
| `--timeout` | | Request timeout in seconds (default: 30) |
| `--verbose` | `-v` | Increase verbosity (-v, -vv, -vvv) |
| `--quiet` | `-q` | Suppress non-essential output |

### Subcommands

```bash
# Configuration management
cli-web-search config init          # Interactive setup
cli-web-search config set KEY VALUE # Set a config value
cli-web-search config get KEY       # Get a config value
cli-web-search config list          # List all configuration
cli-web-search config validate      # Validate API keys
cli-web-search config path          # Show config file path

# Provider management
cli-web-search providers            # List providers and status

# Cache management
cli-web-search cache clear          # Clear the cache
cli-web-search cache stats          # Show cache statistics
```

## Search Providers

| Provider | API Key Required | Notes |
|----------|------------------|-------|
| **Brave** | Yes | High-quality results, good privacy |
| **Google** | Yes | Requires API key + Custom Search Engine ID |
| **DuckDuckGo** | No | Instant Answers API, limited results |
| **Tavily** | Yes | AI-optimized search results |
| **Serper** | Yes | Google results via Serper API |
| **Firecrawl** | Yes | Web crawling and search |

### Getting API Keys

- **Brave Search**: https://brave.com/search/api/
- **Google CSE**: https://developers.google.com/custom-search/v1/overview
- **Tavily**: https://tavily.com/
- **Serper**: https://serper.dev/
- **Firecrawl**: https://firecrawl.dev/

## Configuration

Configuration is stored in `~/.config/cli-web-search/config.yaml`.

### Example Configuration

```yaml
default_provider: brave

providers:
  brave:
    api_key: "your-brave-api-key"
    enabled: true
  google:
    api_key: "your-google-api-key"
    cx: "your-custom-search-engine-id"
    enabled: true
  duckduckgo:
    enabled: true
  tavily:
    api_key: "your-tavily-api-key"
    enabled: true
  serper:
    api_key: "your-serper-api-key"
    enabled: true
  firecrawl:
    api_key: "your-firecrawl-api-key"
    enabled: true

fallback_order:
  - brave
  - google
  - tavily
  - serper
  - firecrawl
  - duckduckgo

defaults:
  num_results: 10
  safe_search: moderate
  timeout: 30
  format: text

cache:
  enabled: true
  ttl_seconds: 3600
  max_entries: 1000
```

### Environment Variables

Environment variables override config file settings:

| Variable | Description |
|----------|-------------|
| `CLI_WEB_SEARCH_BRAVE_API_KEY` | Brave Search API key |
| `CLI_WEB_SEARCH_GOOGLE_API_KEY` | Google CSE API key |
| `CLI_WEB_SEARCH_GOOGLE_CX` | Google Custom Search Engine ID |
| `CLI_WEB_SEARCH_TAVILY_API_KEY` | Tavily API key |
| `CLI_WEB_SEARCH_SERPER_API_KEY` | Serper API key |
| `CLI_WEB_SEARCH_FIRECRAWL_API_KEY` | Firecrawl API key |
| `CLI_WEB_SEARCH_DUCKDUCKGO_ENABLED` | Enable DuckDuckGo (true/false) |
| `CLI_WEB_SEARCH_DEFAULT_PROVIDER` | Default provider name |

## Output Formats

### Text (default)

```
Search Results for: rust programming
Provider: brave | Results: 10 | Time: 245ms

1. The Rust Programming Language
   https://www.rust-lang.org/
   Rust is a language empowering everyone to build reliable and efficient software.

2. Rust (programming language) - Wikipedia
   https://en.wikipedia.org/wiki/Rust_(programming_language)
   Rust is a multi-paradigm, general-purpose programming language...
```

### JSON

```json
{
  "query": "rust programming",
  "provider": "brave",
  "search_time_ms": 245,
  "results": [
    {
      "title": "The Rust Programming Language",
      "url": "https://www.rust-lang.org/",
      "snippet": "Rust is a language empowering everyone...",
      "position": 1
    }
  ]
}
```

### Markdown

```markdown
# Search Results: rust programming

*Provider: brave | 10 results | 245ms*

## 1. The Rust Programming Language
**URL:** https://www.rust-lang.org/

Rust is a language empowering everyone to build reliable and efficient software.

---
```

## Provider Fallback

When a provider fails (rate limit, API error, network issue), cli-web-search automatically:

1. Retries the request with exponential backoff (up to 3 attempts)
2. Respects `Retry-After` headers from rate-limited responses
3. Falls back to the next provider in the configured fallback order

## Use with AI Agents

cli-web-search is designed to work seamlessly with AI coding agents:

```bash
# In your AI agent's tool configuration
cli-web-search -f json "your search query" 2>/dev/null
```

The JSON output provides structured data that's easy for agents to parse and use.

## Troubleshooting

### Common Issues

#### "No search providers configured"

**Problem**: You haven't configured any API keys.

**Solution**: Set up at least one provider:
```bash
# Option 1: Use environment variable
export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"

# Option 2: Use config command
cli-web-search config set providers.brave.api_key "your-api-key"

# Option 3: Enable DuckDuckGo (no API key needed)
cli-web-search config set providers.duckduckgo.enabled true
```

#### "API key not configured" or "Missing API key"

**Problem**: The selected provider doesn't have an API key configured.

**Solution**: Either configure the API key or use a different provider:
```bash
# Check which providers are configured
cli-web-search providers

# Use a different provider
cli-web-search -p duckduckgo "your query"
```

#### Rate limiting (HTTP 429)

**Problem**: You've exceeded the API rate limit for a provider.

**Solution**: The tool automatically retries with exponential backoff and falls back to other providers. You can also:
```bash
# Wait and try again
# Or use a different provider
cli-web-search -p tavily "your query"

# Or disable the rate-limited provider temporarily
cli-web-search config set providers.brave.enabled false
```

#### Empty results from DuckDuckGo

**Problem**: DuckDuckGo's Instant Answer API only returns results for certain types of queries.

**Solution**: DuckDuckGo works best for factual queries. For broader searches, use another provider:
```bash
cli-web-search -p brave "your query"
```

#### Network/Connection errors

**Problem**: Unable to reach the search API.

**Solution**:
```bash
# Increase timeout
cli-web-search --timeout 60 "your query"

# Check with verbose output
cli-web-search -vv "your query"
```

#### Config file permissions

**Problem**: On Unix systems, the config file should have restricted permissions.

**Solution**: The tool automatically sets 600 permissions. If needed:
```bash
chmod 600 ~/.config/cli-web-search/config.yaml
```

### Debugging

Enable verbose output to diagnose issues:

```bash
# Basic verbose
cli-web-search -v "query"

# More verbose
cli-web-search -vv "query"

# Maximum verbosity
cli-web-search -vvv "query"

# With Rust backtrace
RUST_BACKTRACE=1 cli-web-search "query"

# With debug logging
RUST_LOG=debug cli-web-search "query"
```

### Validating Configuration

Check your setup:

```bash
# List all config values
cli-web-search config list

# Validate API keys work
cli-web-search config validate

# Show config file location
cli-web-search config path

# Check provider status
cli-web-search providers
```

## Development

```bash
# Run tests
cargo test

# Run with verbose output
cargo run -- -vv "test query"

# Check code
cargo clippy

# Format code
cargo fmt
```

## License

Apache License 2.0 - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read the AGENTS.md file for development guidelines.
