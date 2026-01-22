# AI Agent Plugins for cli-web-search

This directory contains self-contained, distributable plugins for integrating cli-web-search with AI coding assistants.

## Available Plugins

### 1. OpenCode Plugin

**Directory:** `opencode-cli-web-search/`

A TypeScript plugin that adds web search and URL fetching tools to OpenCode.

- **Package**: `opencode-cli-web-search`
- **Type**: Custom Tools
- **Tools**: `web_search`, `fetch_url`
- **Installation**: npm package or local file
- **Documentation**: [opencode-cli-web-search/README.md](opencode-cli-web-search/README.md)

### 2. Claude Code Plugin

**Directory:** `claude-cli-web-search/`

An Agent Skills plugin that enables Claude to automatically search the web and fetch URLs.

- **Package**: `claude-cli-web-search`
- **Type**: Agent Skills
- **Skills**: `web-search`, `fetch-url`
- **Installation**: Local directory or marketplace
- **Documentation**: [claude-cli-web-search/README.md](claude-cli-web-search/README.md)

## Plugin Structure

### OpenCode Plugin
```
opencode-cli-web-search/
├── package.json         # npm package configuration
├── index.ts             # Plugin implementation
├── README.md            # Installation and usage guide
└── LICENSE              # Apache-2.0 license
```

### Claude Code Plugin
```
claude-cli-web-search/
├── .claude-plugin/
│   └── plugin.json      # Plugin manifest
├── skills/
│   ├── web-search/
│   │   └── SKILL.md     # Web search skill
│   └── fetch-url/
│       └── SKILL.md     # URL fetch skill
├── README.md            # Installation and usage guide
└── LICENSE              # Apache-2.0 license
```

## Prerequisites

Both plugins require:

1. **cli-web-search** installed and in PATH:
   ```bash
   cargo install --git https://github.com/scottgl9/cli-web-search.git
   ```

2. **At least one search provider** configured:
   ```bash
   # Quick option: DuckDuckGo (no API key)
   cli-web-search config set providers.duckduckgo.enabled true
   
   # Or with API key
   export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"
   ```

3. Verify setup:
   ```bash
   cli-web-search --version
   cli-web-search providers
   ```

## Quick Start

### OpenCode

```bash
# Option 1: From npm (when published)
# Add to opencode.json:
{"plugin": ["opencode-cli-web-search"]}

# Option 2: Local installation
cp opencode-cli-web-search/index.ts ~/.config/opencode/plugins/cli-web-search.ts
opencode
```

### Claude Code

```bash
# Option 1: Use directly
cd claude-cli-web-search
claude --plugin-dir .

# Option 2: Global installation
cp -r claude-cli-web-search ~/.claude-plugins/cli-web-search
claude --plugin-dir ~/.claude-plugins/cli-web-search

# Option 3: Marketplace (when published)
/plugin install cli-web-search
```

## Features

Both plugins provide:

- ✅ Web search with 8+ providers
- ✅ Automatic provider fallback
- ✅ Date range filtering
- ✅ Domain include/exclude
- ✅ Safe search levels
- ✅ URL fetching in multiple formats
- ✅ Comprehensive error handling

## Usage Examples

### OpenCode

```typescript
// Ask OpenCode:
"Search for the latest Rust async tutorials"
"Fetch https://tokio.rs/tokio/tutorial and explain it"
"Find recent Tokio updates from this week"
```

### Claude Code

```bash
# Ask Claude:
"Search for Rust error handling best practices"
"Fetch the Tokio documentation and summarize it"
"Find and compare the top 3 Rust web frameworks"
```

## Distribution

### Publishing OpenCode Plugin to npm

```bash
cd opencode-cli-web-search
npm publish
```

Users can then install with:
```json
{
  "plugin": ["opencode-cli-web-search"]
}
```

### Publishing Claude Code Plugin to Marketplace

1. Create a marketplace repository
2. Add `claude-cli-web-search` directory
3. Follow marketplace guidelines
4. Users install with `/plugin install cli-web-search`

See [Claude Code Plugin Marketplaces](https://code.claude.com/docs/en/plugin-marketplaces) for details.

## Development

### Testing OpenCode Plugin Locally

```bash
cd opencode-cli-web-search
# Copy to local plugins
cp index.ts ~/.config/opencode/plugins/cli-web-search.ts
opencode
```

### Testing Claude Code Plugin Locally

```bash
cd claude-cli-web-search
# Run with plugin directory
claude --plugin-dir .
# Check skills loaded
/skills
```

## Validation

Each plugin directory contains everything needed to function independently:

**OpenCode:**
- ✅ Self-contained TypeScript file
- ✅ npm package configuration
- ✅ Complete documentation
- ✅ License file

**Claude Code:**
- ✅ Plugin manifest (plugin.json)
- ✅ All skill files (SKILL.md)
- ✅ Complete documentation
- ✅ License file

## Documentation

- **OpenCode Plugin**: [opencode-cli-web-search/README.md](opencode-cli-web-search/README.md)
- **Claude Code Plugin**: [claude-cli-web-search/README.md](claude-cli-web-search/README.md)
- **Main CLI Tool**: [../README.md](../README.md)
- **Plugin Guide**: [../PLUGINS.md](../PLUGINS.md)
- **Examples**: [../PLUGIN_EXAMPLES.md](../PLUGIN_EXAMPLES.md)

## Support

- **Issues**: https://github.com/scottgl9/cli-web-search/issues
- **Discussions**: https://github.com/scottgl9/cli-web-search/discussions

## License

Apache-2.0 - See LICENSE files in each plugin directory.

## Author

Scott Glover <scottgl@gmail.com>
