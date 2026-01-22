# OpenCode Plugin for cli-web-search

This directory contains the OpenCode plugin for cli-web-search, which provides web search capabilities to OpenCode.

## Features

The plugin adds two custom tools to OpenCode:

### 1. `web_search`

Search the web using cli-web-search with support for multiple providers.

**Parameters:**
- `query` (required): The search query
- `num_results` (optional): Number of results to return (default: 10)
- `provider` (optional): Preferred search provider (brave, google, ddg, tavily, serper, firecrawl, serpapi, bing)
- `date_range` (optional): Filter by date (day, week, month, year)
- `safe_search` (optional): Safe search level (off, moderate, strict)
- `include_domains` (optional): Comma-separated list of domains to include
- `exclude_domains` (optional): Comma-separated list of domains to exclude

**Example usage:**
```
Search the web for "rust async programming best practices"
```

OpenCode will automatically use the `web_search` tool when it needs to find information from the web.

### 2. `fetch_url`

Fetch and extract content from a specific web page.

**Parameters:**
- `url` (required): The URL to fetch
- `format` (optional): Output format (text, html, markdown) - default: markdown
- `max_length` (optional): Maximum content length in bytes

**Example usage:**
```
Fetch and summarize the content from https://rust-lang.github.io/async-book/
```

OpenCode will use the `fetch_url` tool to retrieve the page content.

## Installation

This plugin is automatically loaded when you run OpenCode from this project directory.

To use this plugin in other projects, you can either:

1. **Copy the plugin file** to your global OpenCode plugins directory:
   ```bash
   cp .opencode/plugins/web-search.ts ~/.config/opencode/plugins/
   ```

2. **Or publish as an npm package** (recommended for team sharing):
   ```bash
   # Create a package
   mkdir opencode-cli-web-search
   cd opencode-cli-web-search
   npm init -y
   
   # Copy the plugin
   cp ../.opencode/plugins/web-search.ts index.ts
   
   # Update package.json with proper exports
   # Then publish to npm
   npm publish
   ```

   Then add to your `opencode.json`:
   ```json
   {
     "plugin": ["opencode-cli-web-search"]
   }
   ```

## Requirements

- **cli-web-search** must be installed and available in your PATH
- At least one search provider configured with API keys (or use DuckDuckGo which requires no API key)

### Installing cli-web-search

```bash
# Using cargo
cargo install --git https://github.com/scottgl9/cli-web-search.git

# Or build from source
git clone https://github.com/scottgl9/cli-web-search.git
cd cli-web-search
make build
sudo make install
```

### Configuring API Keys

Set up at least one search provider:

```bash
# Option 1: Use environment variables
export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"

# Option 2: Use the config command
cli-web-search config set providers.brave.api_key "your-api-key"

# Option 3: Enable DuckDuckGo (no API key needed)
cli-web-search config set providers.duckduckgo.enabled true
```

See the [cli-web-search README](../README.md) for more configuration options.

## Usage Examples

Once the plugin is loaded, OpenCode can automatically use these tools. Here are some example prompts:

### Web Search Examples

```
Search for the latest Rust async/await best practices
```

```
Find recent articles about Tokio runtime optimization from the past week
```

```
Search rust-lang.org and docs.rs for information about error handling
```

### URL Fetch Examples

```
Read the content from https://tokio.rs/tokio/tutorial and explain the key concepts
```

```
Fetch https://github.com/tokio-rs/tokio/blob/master/CHANGELOG.md and summarize recent changes
```

```
Get the documentation from https://docs.rs/tokio/latest/tokio/ in markdown format
```

## Troubleshooting

### "cli-web-search not found"

Make sure cli-web-search is installed and in your PATH:
```bash
which cli-web-search
cli-web-search --version
```

### "No search providers configured"

Configure at least one provider or enable DuckDuckGo:
```bash
cli-web-search providers
cli-web-search config set providers.duckduckgo.enabled true
```

### Plugin not loading

Check that the plugin file is in the correct location:
```bash
ls -la .opencode/plugins/web-search.ts
```

For more help, see:
- [OpenCode Plugin Documentation](https://opencode.ai/docs/plugins/)
- [cli-web-search README](../README.md)
