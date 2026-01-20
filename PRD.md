# CLI Web Search Tool - Product Requirements Document

## Overview

A cross-platform command-line web search tool designed to provide AI agents (such as OpenCode, Claude Code, and other CLI-based AI assistants) with seamless web search capabilities. The tool supports multiple search API providers and outputs results in JSON or Markdown format.

## Problem Statement

AI coding agents operating in terminal environments lack a standardized, reliable way to search the web. While some agents have built-in search capabilities, many do not, and there's no universal CLI tool that:
- Works across different AI agent platforms
- Supports multiple search providers
- Provides structured, parseable output
- Handles API key management securely

## Goals

1. **Cross-Platform Support**: Native binaries for Linux and macOS (Windows support planned)
2. **Multi-Provider Support**: Integrate with top search APIs (Google CSE, Brave, DuckDuckGo, etc.)
3. **AI-Agent Friendly**: Output formats optimized for consumption by AI agents
4. **Simple Configuration**: Easy API key setup stored securely in user's home directory
5. **Functional Parity**: Match capabilities of Claude Code's built-in web search

## Target Users

- AI coding agents (OpenCode, Claude Code, Aider, etc.)
- Developers using terminal-based workflows
- Automation scripts requiring web search capabilities
- CLI power users

## Functional Requirements

### Core Features

#### FR-1: Web Search
- Execute web searches via command line
- Support natural language queries
- Return relevant search results with titles, URLs, and snippets

#### FR-2: Multiple Output Formats
- **JSON**: Structured output for programmatic consumption
- **Markdown**: Human-readable formatted output
- **Plain Text**: Simple text output (default)

#### FR-3: Search Provider Support
| Provider | Priority | API Type | Free Tier |
|----------|----------|----------|-----------|
| Brave Search API | P0 | REST | Yes (2,000/month) |
| Google Custom Search Engine | P0 | REST | Yes (100/day) |
| DuckDuckGo Instant Answer | P1 | REST | Yes (limited) |
| Tavily Search API | P1 | REST | Yes (1,000/month) |
| Serper API | P2 | REST | Yes (2,500 queries) |
| SerpAPI | P2 | REST | Yes (100/month) |
| Bing Web Search API | P2 | REST | Yes (1,000/month) |

#### FR-4: Configuration Management
- Store API keys in `~/.config/cli-web-search/config.yaml` (Linux/macOS)
- Support environment variable overrides
- Interactive configuration wizard
- Validate API keys on configuration

#### FR-5: Provider Fallback
- Configure primary and fallback search providers
- Automatic failover on API errors or rate limits
- Configurable retry logic

#### FR-6: Result Filtering
- Limit number of results (default: 10)
- Filter by date range
- Filter by domain (include/exclude)
- Safe search toggle

### Command Interface

```
cli-web-search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  The search query

Options:
  -p, --provider <PROVIDER>    Search provider (brave, google, ddg, tavily, serper)
  -f, --format <FORMAT>        Output format (json, markdown, text) [default: text]
  -n, --num-results <NUM>      Number of results [default: 10]
  -o, --output <FILE>          Write output to file
  --date-range <RANGE>         Filter by date (day, week, month, year)
  --include-domains <DOMAINS>  Only include results from these domains
  --exclude-domains <DOMAINS>  Exclude results from these domains
  --safe-search <LEVEL>        Safe search level (off, moderate, strict)
  --no-cache                   Bypass result cache
  --timeout <SECONDS>          Request timeout [default: 30]
  -v, --verbose                Verbose output
  -q, --quiet                  Suppress non-essential output
  --version                    Print version
  -h, --help                   Print help

Subcommands:
  config      Manage configuration
  providers   List available providers and status
  cache       Manage result cache
```

### Configuration Subcommand

```
cli-web-search config [COMMAND]

Commands:
  init        Interactive configuration setup
  set         Set a configuration value
  get         Get a configuration value
  list        List all configuration
  validate    Validate API keys
  path        Show configuration file path
```

## Non-Functional Requirements

### NFR-1: Performance
- Search response time < 3 seconds (excluding network latency)
- Binary size < 10MB
- Memory usage < 50MB during operation

### NFR-2: Reliability
- Graceful handling of network errors
- Clear error messages with actionable guidance
- Retry logic with exponential backoff

### NFR-3: Security
- API keys stored with restricted file permissions (600)
- No API keys in command history (use config file or env vars)
- Support for credential helpers (future)

### NFR-4: Compatibility
- Linux: x86_64, aarch64
- macOS: x86_64 (Intel), aarch64 (Apple Silicon)
- Minimum glibc version: 2.17

## Technical Architecture

### Technology Stack
- **Language**: Rust (for performance and cross-platform binaries)
- **HTTP Client**: reqwest (async HTTP)
- **CLI Framework**: clap (argument parsing)
- **Serialization**: serde (JSON/YAML)
- **Async Runtime**: tokio

### Project Structure
```
cli-web-search/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI argument parsing
│   ├── config/
│   │   ├── mod.rs           # Configuration module
│   │   ├── loader.rs        # Config file loading
│   │   └── validation.rs    # API key validation
│   ├── providers/
│   │   ├── mod.rs           # Provider trait and registry
│   │   ├── brave.rs         # Brave Search implementation
│   │   ├── google.rs        # Google CSE implementation
│   │   ├── duckduckgo.rs    # DuckDuckGo implementation
│   │   ├── tavily.rs        # Tavily implementation
│   │   └── serper.rs        # Serper implementation
│   ├── output/
│   │   ├── mod.rs           # Output formatting
│   │   ├── json.rs          # JSON formatter
│   │   ├── markdown.rs      # Markdown formatter
│   │   └── text.rs          # Plain text formatter
│   ├── cache/
│   │   ├── mod.rs           # Caching layer
│   │   └── storage.rs       # Cache storage
│   └── error.rs             # Error types
├── tests/
│   ├── integration/         # Integration tests
│   └── providers/           # Provider-specific tests
├── Cargo.toml
├── README.md
├── LICENSE
└── .github/
    └── workflows/
        └── release.yml      # CI/CD for releases
```

### Configuration File Format

```yaml
# ~/.config/cli-web-search/config.yaml

# Default provider to use
default_provider: brave

# Provider configurations
providers:
  brave:
    api_key: "BSA..."
    enabled: true
  google:
    api_key: "AIza..."
    cx: "017576..."  # Custom Search Engine ID
    enabled: true
  tavily:
    api_key: "tvly-..."
    enabled: true
  serper:
    api_key: "..."
    enabled: false

# Fallback chain (used when primary fails)
fallback_order:
  - brave
  - google
  - tavily

# Default options
defaults:
  num_results: 10
  safe_search: moderate
  timeout: 30
  format: text

# Cache settings
cache:
  enabled: true
  ttl_seconds: 3600
  max_entries: 1000
```

### Output Formats

#### JSON Output
```json
{
  "query": "rust async programming",
  "provider": "brave",
  "timestamp": "2024-01-15T10:30:00Z",
  "results": [
    {
      "title": "Asynchronous Programming in Rust",
      "url": "https://rust-lang.github.io/async-book/",
      "snippet": "This book aims to be a comprehensive guide to async programming in Rust...",
      "position": 1
    }
  ],
  "total_results": 10,
  "search_time_ms": 245
}
```

#### Markdown Output
```markdown
# Search Results: rust async programming

*Provider: Brave | Results: 10 | Time: 245ms*

---

## 1. Asynchronous Programming in Rust
**URL:** https://rust-lang.github.io/async-book/

This book aims to be a comprehensive guide to async programming in Rust...

---
```

## Success Metrics

1. **Adoption**: Used by 3+ AI agent projects within 6 months
2. **Reliability**: 99.9% uptime for API calls (excluding provider issues)
3. **Performance**: P95 response time < 2 seconds
4. **Coverage**: Support for top 5 search APIs

## Milestones

### Phase 1: MVP (Weeks 1-3)
- [ ] Core CLI structure
- [ ] Brave Search integration
- [ ] Google CSE integration
- [ ] JSON and Markdown output
- [ ] Basic configuration management

### Phase 2: Enhanced Features (Weeks 4-5)
- [ ] Additional providers (DuckDuckGo, Tavily)
- [ ] Provider fallback logic
- [ ] Result caching
- [ ] Date and domain filtering

### Phase 3: Polish (Weeks 6-7)
- [ ] Configuration wizard
- [ ] Comprehensive error handling
- [ ] Documentation
- [ ] CI/CD pipeline
- [ ] Binary releases

### Phase 4: Extended Support (Future)
- [ ] Windows support
- [ ] Additional providers
- [ ] Plugin system for custom providers
- [ ] MCP server mode

## Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| API rate limits | High | Medium | Implement caching, fallback providers |
| Provider API changes | Medium | Medium | Abstract provider interface, version pinning |
| Cross-platform issues | Medium | Low | CI testing on all platforms |
| Security vulnerabilities | High | Low | Security audit, dependency scanning |

## Appendix

### Competitive Analysis

| Feature | cli-web-search | ddgr | googler | s (web search) |
|---------|----------------|------|---------|----------------|
| Multi-provider | Yes | No | No | Limited |
| JSON output | Yes | Yes | Yes | No |
| API-based | Yes | Scraping | Scraping | Mixed |
| AI-agent optimized | Yes | No | No | No |
| Active maintenance | Planned | Limited | Limited | No |

### API Documentation References
- [Brave Search API](https://brave.com/search/api/)
- [Google Custom Search JSON API](https://developers.google.com/custom-search/v1/overview)
- [DuckDuckGo Instant Answer API](https://duckduckgo.com/api)
- [Tavily Search API](https://tavily.com/)
- [Serper API](https://serper.dev/)
