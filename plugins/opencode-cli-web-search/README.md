# opencode-cli-web-search

OpenCode plugin for web search and URL fetching using [cli-web-search](https://github.com/scottgl9/cli-web-search).

## Installation

### Prerequisites

1. **cli-web-search** must be installed and available in your PATH:
   ```bash
   cargo install --git https://github.com/scottgl9/cli-web-search.git
   ```

2. **Configure at least one search provider**:
   ```bash
   # Quick option: Enable DuckDuckGo (no API key needed)
   cli-web-search config set providers.duckduckgo.enabled true
   
   # Or use a provider with API key
   export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"
   ```

3. Verify setup:
   ```bash
   cli-web-search --version
   cli-web-search providers
   ```

### Install Plugin

#### Option 1: Via npm (when published)

Add to your `opencode.json`:
```json
{
  "plugin": ["opencode-cli-web-search"]
}
```

#### Option 2: Local file

Copy `index.ts` to your OpenCode plugins directory:
```bash
cp index.ts ~/.config/opencode/plugins/cli-web-search.ts
```

#### Option 3: From project directory

When working in the cli-web-search repository:
```bash
cd /path/to/cli-web-search
opencode
```

The plugin is automatically loaded from `.opencode/plugins/`.

## Features

This plugin adds two custom tools to OpenCode:

### `web_search`

Search the web with comprehensive options.

**Parameters:**
- `query` (required): The search query
- `num_results` (optional): Number of results (default: 10)
- `provider` (optional): Search provider (brave, google, ddg, tavily, serper, firecrawl, serpapi, bing)
- `date_range` (optional): Filter by date (day, week, month, year)
- `safe_search` (optional): Safe search level (off, moderate, strict)
- `include_domains` (optional): Comma-separated domains to include
- `exclude_domains` (optional): Comma-separated domains to exclude

**Example:**
```
Search for recent Rust async tutorials from this week
```

OpenCode will automatically use the `web_search` tool.

### `fetch_url`

Fetch and extract content from web pages.

**Parameters:**
- `url` (required): The URL to fetch
- `format` (optional): Output format (text, html, markdown) - default: markdown
- `max_length` (optional): Maximum content length in bytes

**Example:**
```
Fetch the documentation from https://tokio.rs and explain it
```

OpenCode will automatically use the `fetch_url` tool.

## Usage Examples

### Basic Web Search
```
"Search for the latest Tokio updates"
```

### Filtered Search
```
"Find Rust documentation about error handling from rust-lang.org and docs.rs"
```

### Recent Information
```
"Search for articles about async Rust from the past week"
```

### Fetch and Analyze
```
"Fetch https://docs.rs/tokio/latest/tokio/ and summarize the key features"
```

### Combined Workflow
```
"Search for Tokio tutorials, then fetch and summarize the top 3 results"
```

## Configuration

The plugin uses cli-web-search configuration. Customize via:

```bash
# View configuration
cli-web-search config list

# Set default provider
cli-web-search config set default_provider brave

# Configure provider API key
cli-web-search config set providers.brave.api_key "your-key"

# Set fallback order
cli-web-search config set fallback_order "brave,google,duckduckgo"
```

Configuration file location: `~/.config/cli-web-search/config.yaml`

## Supported Providers

| Provider | API Key Required | Notes |
|----------|------------------|-------|
| Brave | Yes | High-quality results |
| Google CSE | Yes | Requires API key + CX |
| DuckDuckGo | No | Free, limited coverage |
| Tavily | Yes | AI-optimized results |
| Serper | Yes | Google via Serper API |
| Firecrawl | Yes | Web crawling |
| SerpAPI | Yes | Multiple search engines |
| Bing | Yes | Microsoft Bing API |

Get API keys:
- [Brave Search](https://brave.com/search/api/)
- [Google CSE](https://developers.google.com/custom-search/v1/overview)
- [Tavily](https://tavily.com/)
- [Serper](https://serper.dev/)
- [Firecrawl](https://firecrawl.dev/)
- [SerpAPI](https://serpapi.com/)
- [Bing](https://azure.microsoft.com/en-us/products/ai-services/ai-search)

## Troubleshooting

### "cli-web-search not found"

Ensure cli-web-search is installed and in PATH:
```bash
which cli-web-search
cli-web-search --version
```

### "No search providers configured"

Configure at least one provider:
```bash
cli-web-search providers
cli-web-search config set providers.duckduckgo.enabled true
```

### Plugin not loading

Check the plugin file location:
```bash
ls -la ~/.config/opencode/plugins/cli-web-search.ts
```

## Development

To modify the plugin:

1. Clone the repository:
   ```bash
   git clone https://github.com/scottgl9/cli-web-search.git
   cd cli-web-search/plugins/opencode-cli-web-search
   ```

2. Edit `index.ts`

3. Test locally:
   ```bash
   cp index.ts ~/.config/opencode/plugins/cli-web-search.ts
   opencode
   ```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) file for details.

## Links

- [cli-web-search](https://github.com/scottgl9/cli-web-search)
- [OpenCode](https://opencode.ai/)
- [OpenCode Plugin Documentation](https://opencode.ai/docs/plugins/)

## Support

- Issues: https://github.com/scottgl9/cli-web-search/issues
- Discussions: https://github.com/scottgl9/cli-web-search/discussions
