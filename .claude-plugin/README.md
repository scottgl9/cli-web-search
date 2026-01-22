# Claude Code Plugin for cli-web-search

This directory contains the Claude Code plugin for cli-web-search, which provides web search and URL fetching capabilities to Claude Code.

## Features

This plugin adds two Agent Skills to Claude Code:

### 1. **web-search** - Search the Web

Search the web using cli-web-search with support for multiple providers and automatic fallback.

**Capabilities:**
- Search using multiple providers (Brave, Google, DuckDuckGo, Tavily, Serper, Firecrawl, SerpAPI, Bing)
- Filter results by date range (day, week, month, year)
- Domain filtering (include/exclude specific domains)
- Configurable number of results
- Safe search levels
- Automatic provider fallback

**When Claude uses this skill:**
- When you ask Claude to search for information
- When Claude needs current information beyond its training data
- When researching programming topics, libraries, or frameworks
- When verifying technical facts or best practices

### 2. **fetch-url** - Fetch Web Page Content

Fetch and extract content from specific web pages in readable formats.

**Capabilities:**
- Convert web pages to text, markdown, or HTML
- Extract content with size limits
- Get page metadata
- Handle various content types

**When Claude uses this skill:**
- After finding relevant URLs in search results
- When you ask Claude to read a specific web page
- When accessing documentation or articles
- When you provide a URL and ask for analysis

## Installation

### Option 1: Project-Level Plugin (Recommended for this project)

This plugin is already set up in this project. Just ensure cli-web-search is installed (see Requirements below).

To test the plugin:

```bash
# Navigate to the project directory
cd /path/to/cli-web-search

# Start Claude Code with the plugin
claude --plugin-dir .
```

### Option 2: Copy to Global Plugin Directory

To use this plugin across all your projects:

```bash
# Copy the entire plugin directory
cp -r .claude-plugin ~/.claude-plugins/cli-web-search
cp -r skills ~/.claude-plugins/cli-web-search/

# Or create a symlink
ln -s "$(pwd)/.claude-plugin" ~/.claude-plugins/cli-web-search/.claude-plugin
ln -s "$(pwd)/skills" ~/.claude-plugins/cli-web-search/skills
```

### Option 3: Install from Marketplace

Once published to a marketplace:

```bash
claude
# Then in Claude Code:
/plugin install cli-web-search
```

## Requirements

### 1. Install cli-web-search

The plugin requires cli-web-search to be installed and available in your PATH.

**Using Cargo:**
```bash
cargo install --git https://github.com/scottgl9/cli-web-search.git
```

**From Source:**
```bash
git clone https://github.com/scottgl9/cli-web-search.git
cd cli-web-search
make build
sudo make install
```

**Using Debian Package:**
```bash
# If available
sudo dpkg -i cli-web-search_*.deb
```

Verify installation:
```bash
cli-web-search --version
```

### 2. Configure Search Providers

Set up at least one search provider:

**Option 1: Environment Variables**
```bash
export CLI_WEB_SEARCH_BRAVE_API_KEY="your-api-key"
export CLI_WEB_SEARCH_GOOGLE_API_KEY="your-api-key"
export CLI_WEB_SEARCH_GOOGLE_CX="your-custom-search-engine-id"
# ... other providers
```

**Option 2: Configuration File**
```bash
cli-web-search config init  # Interactive setup
# Or manually set values:
cli-web-search config set providers.brave.api_key "your-api-key"
```

**Option 3: Use DuckDuckGo (No API Key Required)**
```bash
cli-web-search config set providers.duckduckgo.enabled true
```

Check provider status:
```bash
cli-web-search providers
```

## Usage

Once the plugin is loaded, Claude Code automatically has access to these skills. Claude will invoke them when appropriate, or you can explicitly reference them.

### Automatic Usage Examples

Simply ask Claude:

```
Search for the latest Rust async/await best practices
```

```
Find recent articles about Tokio runtime from the past week
```

```
Read the documentation from https://tokio.rs/tokio/tutorial and explain it
```

```
Search rust-lang.org for information about lifetimes, then fetch and summarize the official docs
```

Claude will automatically use the appropriate skill (web-search or fetch-url) based on your request.

### Check Available Skills

To see loaded skills:
```bash
/skills
```

### Skill Invocation (Advanced)

While Claude automatically uses these skills, you can also reference them explicitly:

```
Use the web-search skill to find information about Rust error handling
```

```
Use the fetch-url skill to get content from https://docs.rs/tokio
```

## Configuration

### Plugin Settings

The plugin uses cli-web-search's configuration system. Customize behavior by editing:

**Global Config:**
```bash
~/.config/cli-web-search/config.yaml
```

**Example Configuration:**
```yaml
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

### Skill Customization

The skills are defined in:
- `skills/web-search/SKILL.md` - Web search skill
- `skills/fetch-url/SKILL.md` - URL fetch skill

You can modify these files to customize:
- When Claude uses each skill (description field)
- Instructions for how to use the skills
- Default parameters
- Error handling behavior

## Examples

### Research a Topic

```
I need to understand how Tokio's runtime works. Search for recent articles and tutorials.
```

Claude will:
1. Use web-search to find relevant articles
2. Present the results with URLs
3. Optionally fetch and summarize specific URLs

### Get Official Documentation

```
Fetch the Tokio tutorial from tokio.rs and summarize the key concepts
```

Claude will:
1. Use fetch-url to get the page content
2. Extract and analyze the documentation
3. Provide a summary

### Combined Workflow

```
Search for "rust async programming best practices" and then read the top 2 results
```

Claude will:
1. Use web-search to find articles
2. Use fetch-url to retrieve content from the best results
3. Synthesize information from multiple sources

### Filtered Search

```
Search for Rust documentation on error handling, but only from rust-lang.org and docs.rs
```

Claude will use web-search with domain filtering.

## Troubleshooting

### "cli-web-search not found" or "command not found"

**Problem:** The cli-web-search binary is not in the PATH.

**Solution:**
```bash
# Check if installed
which cli-web-search

# If not found, install it
cargo install --git https://github.com/scottgl9/cli-web-search.git

# Or add to PATH if installed in a custom location
export PATH="$PATH:/path/to/cli-web-search"
```

### "No search providers configured"

**Problem:** No search provider API keys are configured.

**Solution:**
```bash
# Check provider status
cli-web-search providers

# Enable DuckDuckGo (no API key needed)
cli-web-search config set providers.duckduckgo.enabled true

# Or configure a provider with API key
cli-web-search config set providers.brave.api_key "your-api-key"
```

### Skills Not Loading

**Problem:** Claude doesn't recognize the web-search or fetch-url skills.

**Solution:**
```bash
# Verify plugin structure
ls -la .claude-plugin/plugin.json
ls -la skills/web-search/SKILL.md
ls -la skills/fetch-url/SKILL.md

# Check plugin is loaded
claude --plugin-dir .
# Then run: /skills
```

### Search Returns No Results

**Problem:** Searches complete but return no results.

**Possible causes:**
- DuckDuckGo has limited coverage for some queries
- Rate limiting from the API provider
- Network issues

**Solution:**
```bash
# Try a different provider
cli-web-search -p brave "your query"

# Check with verbose output
cli-web-search -vv "your query"

# Verify API keys are valid
cli-web-search config validate
```

### API Rate Limiting

**Problem:** Getting rate limit errors (HTTP 429).

**Solution:**
- The tool automatically retries with exponential backoff
- Configure multiple providers for automatic fallback
- Wait before retrying
- Upgrade to a higher API tier if needed

## Plugin Structure

```
cli-web-search/
├── .claude-plugin/
│   └── plugin.json          # Plugin manifest
└── skills/
    ├── web-search/
    │   └── SKILL.md          # Web search skill definition
    └── fetch-url/
        └── SKILL.md          # URL fetch skill definition
```

## Development

### Testing Changes

After modifying skill files:

```bash
# Restart Claude Code to reload skills
claude --plugin-dir .
```

### Customizing Skills

Edit the SKILL.md files to:
- Change when Claude invokes each skill (modify `description` in frontmatter)
- Update instructions or examples
- Add domain-specific knowledge

### Creating Additional Skills

Add new skills by creating additional directories under `skills/`:

```bash
mkdir skills/my-custom-skill
touch skills/my-custom-skill/SKILL.md
```

## Resources

- **cli-web-search Documentation:** See the main [README.md](../README.md)
- **Claude Code Plugin Docs:** https://code.claude.com/docs/en/plugins
- **Agent Skills Guide:** https://code.claude.com/docs/en/skills
- **Search Provider APIs:**
  - Brave Search: https://brave.com/search/api/
  - Google CSE: https://developers.google.com/custom-search/v1/overview
  - Tavily: https://tavily.com/
  - Serper: https://serper.dev/

## Contributing

To improve this plugin:

1. Test your changes thoroughly
2. Update documentation
3. Consider backward compatibility
4. Submit issues or PRs to the main repository

## License

Apache License 2.0 - Same as cli-web-search

## Author

Scott Glover <scottgl@gmail.com>

## Version History

- **1.0.0** (2026-01-21): Initial release
  - Web search skill with multiple provider support
  - URL fetch skill with multiple format support
  - Comprehensive documentation and examples
