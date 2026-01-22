# Quick Reference: AI Agent Plugins

This is a quick reference for using the cli-web-search plugins with OpenCode and Claude Code.

## Installation

### Prerequisites

```bash
# Install cli-web-search
cargo install --git https://github.com/scottgl9/cli-web-search.git

# Configure a provider (choose one)
export CLI_WEB_SEARCH_BRAVE_API_KEY="your-key"  # Brave Search
cli-web-search config set providers.duckduckgo.enabled true  # DuckDuckGo (free)

# Verify
cli-web-search --version
cli-web-search providers
```

## OpenCode

### Setup
```bash
cd cli-web-search
opencode
```

### Usage
OpenCode automatically has access to these tools:

- `web_search` - Search the web
- `fetch_url` - Fetch web page content

Just ask OpenCode to search for information, and it will use these tools automatically.

### Example Prompts
```
"Search for the latest Rust async best practices"
"Fetch the documentation from https://tokio.rs"
"Find recent articles about Tokio runtime from the past week"
```

## Claude Code

### Setup
```bash
cd cli-web-search
claude --plugin-dir .
```

### Usage
Claude automatically has access to these skills:

- `web-search` - Search the web
- `fetch-url` - Fetch web page content

Claude invokes these skills automatically when appropriate.

### Example Prompts
```
"Search for Rust error handling best practices"
"Read the content from https://docs.rs/tokio and explain it"
"Find and summarize the top 3 Tokio tutorials"
```

### Check Skills
```bash
# Inside Claude Code
/skills
```

## Common Commands

### Test CLI Directly
```bash
# Basic search
cli-web-search "rust programming"

# JSON output (for agents)
cli-web-search -f json "rust async"

# Fetch a URL
cli-web-search fetch "https://example.com" --stdout

# With specific provider
cli-web-search -p brave "rust tutorials"
```

### Configuration
```bash
# Check providers
cli-web-search providers

# Set API key
cli-web-search config set providers.brave.api_key "key"

# View config
cli-web-search config list

# Config file location
cli-web-search config path
```

## Troubleshooting

### Plugin Not Working

```bash
# Check cli-web-search is installed
which cli-web-search
cli-web-search --version

# Check providers are configured
cli-web-search providers

# Test search directly
cli-web-search -f json "test"
```

### OpenCode Specific

```bash
# Check plugin file exists
ls -la .opencode/plugins/web-search.ts

# Check logs for errors
# (OpenCode shows errors in the UI)
```

### Claude Code Specific

```bash
# Check plugin structure
ls -la .claude-plugin/plugin.json
ls -la skills/*/SKILL.md

# Verify JSON is valid
jq . .claude-plugin/plugin.json

# Check skills are loaded
claude --plugin-dir .
# Then run: /skills
```

## Advanced Usage

### Multiple Providers
```yaml
# ~/.config/cli-web-search/config.yaml
fallback_order:
  - brave
  - google
  - duckduckgo
```

### Custom Parameters

**OpenCode** - Modify `.opencode/plugins/web-search.ts`

**Claude Code** - Modify skill files:
- `skills/web-search/SKILL.md`
- `skills/fetch-url/SKILL.md`

## Documentation

- Full docs: [PLUGINS.md](PLUGINS.md)
- OpenCode: [.opencode/README.md](.opencode/README.md)
- Claude Code: [.claude-plugin/README.md](.claude-plugin/README.md)
- CLI docs: [README.md](README.md)
