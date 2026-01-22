# claude-cli-web-search

Claude Code plugin for web search and URL fetching using [cli-web-search](https://github.com/scottgl9/cli-web-search).

This plugin provides Agent Skills that Claude automatically uses to search the web and fetch web page content.

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

#### Option 1: From local directory (during development)

```bash
# Clone or download this plugin directory
cd /path/to/claude-cli-web-search

# Run Claude Code with the plugin
claude --plugin-dir .
```

#### Option 2: Copy to global plugins directory

```bash
# Copy the entire plugin directory
cp -r /path/to/claude-cli-web-search ~/.claude-plugins/cli-web-search
```

Then in any project:
```bash
claude --plugin-dir ~/.claude-plugins/cli-web-search
```

#### Option 3: Via marketplace (when published)

```bash
claude
# Then in Claude Code:
/plugin install cli-web-search
```

## Features

This plugin adds two Agent Skills that Claude automatically invokes:

### `web-search`

Search the web with multiple providers and comprehensive filtering options.

**Capabilities:**
- Search with 8+ providers (Brave, Google, DuckDuckGo, Tavily, Serper, etc.)
- Date range filtering (day, week, month, year)
- Domain include/exclude filtering
- Safe search levels
- Automatic provider fallback

**When Claude uses this:**
- When you ask for current information
- When researching topics
- When looking up documentation
- When verifying facts

### `fetch-url`

Fetch and extract content from web pages in readable formats.

**Capabilities:**
- Convert pages to text, HTML, or markdown
- Handle content size limits
- Extract page metadata
- Multiple timeout options

**When Claude uses this:**
- After finding URLs in search results
- When you provide a URL to analyze
- When accessing documentation
- When reading articles or posts

## Usage

Once loaded, Claude automatically has access to these skills. Just ask naturally:

### Examples

```
"Search for the latest Rust async best practices"
```

```
"Find recent articles about Tokio from the past week"
```

```
"Fetch the Tokio tutorial from tokio.rs and explain it"
```

```
"Search rust-lang.org for information about lifetimes"
```

```
"Find the top 3 Tokio tutorials and summarize each"
```

Claude will automatically use the appropriate skill based on your request.

## Plugin Structure

```
claude-cli-web-search/
├── .claude-plugin/
│   └── plugin.json          # Plugin manifest
├── skills/
│   ├── web-search/
│   │   └── SKILL.md         # Web search skill
│   └── fetch-url/
│       └── SKILL.md         # URL fetch skill
├── README.md                # This file
└── LICENSE                  # Apache-2.0 license
```

## Configuration

The plugin uses cli-web-search configuration:

```bash
# View all configuration
cli-web-search config list

# Set default provider
cli-web-search config set default_provider brave

# Configure fallback order
# Edit ~/.config/cli-web-search/config.yaml:
fallback_order:
  - brave
  - google
  - duckduckgo
```

### Example Configuration

```yaml
# ~/.config/cli-web-search/config.yaml
default_provider: brave

providers:
  brave:
    api_key: "your-api-key"
    enabled: true
  duckduckgo:
    enabled: true

fallback_order:
  - brave
  - duckduckgo

defaults:
  num_results: 10
  safe_search: moderate
  timeout: 30

cache:
  enabled: true
  ttl_seconds: 3600
```

## Customization

### Modify When Skills Are Invoked

Edit the `description` field in the skill's frontmatter:

```markdown
---
name: web-search
description: Your custom description that tells Claude when to use this skill
---
```

### Add Domain-Specific Knowledge

Add examples and best practices to the skill content:

```markdown
## Examples for Your Domain

### Search for API Documentation
\```bash
cli-web-search --include-domains "docs.rs,rust-lang.org" "rust async API"
\```
```

## Supported Search Providers

| Provider | API Key | Get Key |
|----------|---------|---------|
| Brave | Required | [brave.com/search/api](https://brave.com/search/api/) |
| Google CSE | Required | [developers.google.com](https://developers.google.com/custom-search/v1/overview) |
| DuckDuckGo | **None** | Free, no registration |
| Tavily | Required | [tavily.com](https://tavily.com/) |
| Serper | Required | [serper.dev](https://serper.dev/) |
| Firecrawl | Required | [firecrawl.dev](https://firecrawl.dev/) |
| SerpAPI | Required | [serpapi.com](https://serpapi.com/) |
| Bing | Required | [azure.microsoft.com](https://azure.microsoft.com/en-us/products/ai-services/ai-search) |

## Troubleshooting

### Skills Not Loading

```bash
# Verify plugin structure
ls -la .claude-plugin/plugin.json
ls -la skills/*/SKILL.md

# Check JSON is valid
jq . .claude-plugin/plugin.json

# Verify skills are loaded
claude --plugin-dir /path/to/plugin
# Then run: /skills
```

### "cli-web-search not found"

```bash
# Check if installed
which cli-web-search

# Install if missing
cargo install --git https://github.com/scottgl9/cli-web-search.git
```

### "No search providers configured"

```bash
# Check provider status
cli-web-search providers

# Enable DuckDuckGo (no API key)
cli-web-search config set providers.duckduckgo.enabled true

# Or configure API key
cli-web-search config set providers.brave.api_key "your-key"
```

### No Results from Search

```bash
# Try with different provider
cli-web-search -p brave "test query"

# Check with verbose output
cli-web-search -vv "test query"

# Validate configuration
cli-web-search config validate
```

## Development

To modify the plugin:

1. Clone the repository:
   ```bash
   git clone https://github.com/scottgl9/cli-web-search.git
   cd cli-web-search/plugins/claude-cli-web-search
   ```

2. Edit skill files:
   - `skills/web-search/SKILL.md`
   - `skills/fetch-url/SKILL.md`

3. Test changes:
   ```bash
   claude --plugin-dir .
   ```

4. Validate:
   ```bash
   # Check JSON syntax
   jq . .claude-plugin/plugin.json
   
   # Verify skills have frontmatter
   head -10 skills/*/SKILL.md
   ```

## Publishing

To publish this plugin to a marketplace:

1. Ensure all files are included
2. Update version in `plugin.json`
3. Follow marketplace guidelines
4. Submit plugin for review

## Version History

- **1.0.0** - Initial release
  - Web search skill with multiple providers
  - URL fetch skill with format support
  - Comprehensive documentation

## License

Apache-2.0 - See [LICENSE](LICENSE) file for details.

## Links

- **cli-web-search**: https://github.com/scottgl9/cli-web-search
- **Claude Code**: https://code.claude.com
- **Plugin Docs**: https://code.claude.com/docs/en/plugins
- **Skills Guide**: https://code.claude.com/docs/en/skills

## Support

- **Issues**: https://github.com/scottgl9/cli-web-search/issues
- **Discussions**: https://github.com/scottgl9/cli-web-search/discussions
- **Discord**: [Claude Developers Discord](https://anthropic.com/discord)

## Contributing

Contributions welcome! Please:

1. Test changes thoroughly
2. Update documentation
3. Follow existing code style
4. Submit PR with description

## Author

Scott Glover <scottgl@gmail.com>
