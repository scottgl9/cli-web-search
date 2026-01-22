# AI Agent Plugins for cli-web-search

This repository includes plugins for integrating cli-web-search with popular AI coding assistants.

## Available Plugins

### 1. OpenCode Plugin

**Location:** `.opencode/plugins/web-search.ts`

Provides custom tools for OpenCode that enable web search and URL fetching capabilities.

**Features:**
- `web_search` tool - Search the web with multiple providers
- `fetch_url` tool - Fetch and extract web page content
- Full parameter support (providers, date ranges, domain filtering, etc.)
- Structured logging and error handling

**Documentation:** [.opencode/README.md](.opencode/README.md)

### 2. Claude Code Plugin

**Location:** `.claude-plugin/` and `skills/`

Provides Agent Skills for Claude Code that enable web search and URL fetching.

**Features:**
- `web-search` skill - Search the web automatically
- `fetch-url` skill - Fetch web page content automatically
- Model-invoked skills that Claude uses when appropriate
- Comprehensive examples and usage patterns

**Documentation:** [.claude-plugin/README.md](.claude-plugin/README.md)

## Quick Start

### For OpenCode Users

1. Install cli-web-search (see main [README.md](README.md))
2. Configure at least one search provider
3. The plugin is automatically loaded when running OpenCode from this directory

```bash
cd cli-web-search
opencode
# Now OpenCode has web_search and fetch_url tools available
```

### For Claude Code Users

1. Install cli-web-search (see main [README.md](README.md))
2. Configure at least one search provider
3. Run Claude Code with the plugin:

```bash
cd cli-web-search
claude --plugin-dir .
# Now Claude has web-search and fetch-url skills available
```

## Comparison

| Feature | OpenCode Plugin | Claude Code Plugin |
|---------|----------------|-------------------|
| **Type** | Custom Tools | Agent Skills |
| **Invocation** | Explicit tool calls by OpenCode | Automatic based on context |
| **Configuration** | TypeScript file | JSON manifest + Markdown |
| **Search** | `web_search` tool | `web-search` skill |
| **Fetch** | `fetch_url` tool | `fetch-url` skill |
| **Location** | `.opencode/plugins/` | `.claude-plugin/` + `skills/` |

## Plugin Capabilities

Both plugins provide the same core functionality:

### Web Search
- Multiple provider support (Brave, Google, DuckDuckGo, Tavily, Serper, Firecrawl, SerpAPI, Bing)
- Automatic provider fallback
- Date range filtering
- Domain include/exclude
- Safe search levels
- Configurable result count

### URL Fetching
- Convert web pages to text, markdown, or HTML
- Content size limits
- Metadata extraction
- Timeout configuration

## Installation Requirements

Both plugins require:

1. **cli-web-search** installed and in PATH
   ```bash
   cargo install --git https://github.com/scottgl9/cli-web-search.git
   ```

2. **At least one search provider configured**
   ```bash
   # Easy option: Enable DuckDuckGo (no API key)
   cli-web-search config set providers.duckduckgo.enabled true
   
   # Or configure a provider with API key
   export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"
   ```

3. **Verify setup**
   ```bash
   cli-web-search --version
   cli-web-search providers
   ```

## Using Plugins in Other Projects

### OpenCode

Copy the plugin to your global plugins directory:
```bash
cp .opencode/plugins/web-search.ts ~/.config/opencode/plugins/
```

Or publish as an npm package and add to your `opencode.json`:
```json
{
  "plugin": ["opencode-cli-web-search"]
}
```

### Claude Code

Copy the plugin directory:
```bash
cp -r .claude-plugin ~/.claude-plugins/cli-web-search
cp -r skills ~/.claude-plugins/cli-web-search/
```

Or install from a marketplace when published:
```bash
/plugin install cli-web-search
```

## Example Usage

### OpenCode

```typescript
// OpenCode automatically uses these tools when appropriate

// Example prompt:
"Search for the latest Rust async/await tutorials"

// OpenCode will call web_search tool with the query
```

### Claude Code

```bash
# Claude automatically uses skills when appropriate

# Example prompts:
"Search for recent Tokio updates"
"Fetch the content from https://tokio.rs/tokio/tutorial"
"Find and summarize the top 3 articles about Rust error handling"
```

## Documentation

- **Main CLI Documentation:** [README.md](README.md)
- **OpenCode Plugin:** [.opencode/README.md](.opencode/README.md)
- **Claude Code Plugin:** [.claude-plugin/README.md](.claude-plugin/README.md)
- **OpenCode Docs:** https://opencode.ai/docs/plugins/
- **Claude Code Docs:** https://code.claude.com/docs/en/plugins

## Troubleshooting

### Plugin Not Loading

**OpenCode:**
```bash
# Check plugin file exists
ls -la .opencode/plugins/web-search.ts

# Verify TypeScript syntax
npx tsc --noEmit .opencode/plugins/web-search.ts
```

**Claude Code:**
```bash
# Check plugin structure
ls -la .claude-plugin/plugin.json
ls -la skills/*/SKILL.md

# Verify JSON syntax
jq . .claude-plugin/plugin.json

# Test plugin loading
claude --plugin-dir .
```

### cli-web-search Not Found

```bash
# Check if installed
which cli-web-search

# Check version
cli-web-search --version

# If not found, install it
cargo install --git https://github.com/scottgl9/cli-web-search.git
```

### No Search Results

```bash
# Check provider configuration
cli-web-search providers

# Test search directly
cli-web-search -f json "test query"

# Enable a provider if none configured
cli-web-search config set providers.duckduckgo.enabled true
```

## Contributing

Contributions to improve either plugin are welcome! Please:

1. Test changes thoroughly
2. Update relevant documentation
3. Follow the coding style of each plugin
4. Submit issues or PRs to the repository

## License

Apache License 2.0 - See [LICENSE](LICENSE) file for details

## Author

Scott Glover <scottgl@gmail.com>
