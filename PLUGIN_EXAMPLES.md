# Plugin Usage Examples

This document provides practical examples of using the cli-web-search plugins with OpenCode and Claude Code.

## Setup

First, ensure cli-web-search is installed and configured:

```bash
# Install cli-web-search
cargo install --git https://github.com/scottgl9/cli-web-search.git

# Configure a provider (quick option: DuckDuckGo, no API key needed)
cli-web-search config set providers.duckduckgo.enabled true

# Verify setup
cli-web-search --version
cli-web-search providers
```

## OpenCode Examples

### Starting OpenCode with Plugin

```bash
cd /path/to/cli-web-search
opencode
```

### Example 1: Basic Web Search

**User:** "Search for the latest Rust async/await best practices"

**How it works:**
- OpenCode recognizes the need to search the web
- Calls the `web_search` tool with query "Rust async/await best practices"
- Returns structured JSON results with titles, URLs, and snippets
- Presents results to you in a readable format

### Example 2: Recent Information with Date Filter

**User:** "Find articles about Tokio runtime updates from the past week"

**How it works:**
- OpenCode uses `web_search` with `date_range: "week"`
- Filters results to recent content only
- Shows latest news and updates

### Example 3: Domain-Filtered Search

**User:** "Search rust-lang.org and docs.rs for information about error handling"

**How it works:**
- OpenCode uses `web_search` with `include_domains: "rust-lang.org,docs.rs"`
- Returns only results from official sources
- Ensures high-quality, authoritative results

### Example 4: Fetch and Analyze URL

**User:** "Fetch the content from https://tokio.rs/tokio/tutorial and explain the key concepts"

**How it works:**
- OpenCode uses `fetch_url` to get the page content in markdown format
- Analyzes the tutorial content
- Explains key concepts in a clear, structured way

### Example 5: Multi-Step Workflow

**User:** "Find the top 3 Tokio tutorials and summarize each one"

**How it works:**
1. Calls `web_search` to find Tokio tutorials
2. Identifies top 3 most relevant results
3. Calls `fetch_url` for each URL
4. Summarizes content from each tutorial
5. Presents combined summary

## Claude Code Examples

### Starting Claude Code with Plugin

```bash
cd /path/to/cli-web-search
claude --plugin-dir .
```

### Example 1: Automatic Web Search

**User:** "I need to understand how Tokio's runtime scheduler works"

**How it works:**
- Claude recognizes need for current information
- Automatically invokes `web-search` skill
- Searches for relevant documentation and articles
- Synthesizes information from multiple sources
- Provides comprehensive explanation

### Example 2: Research with Follow-up

**User:** "What are the best practices for Rust error handling?"

**How it works:**
1. Claude uses `web-search` to find articles
2. Identifies most authoritative sources
3. Uses `fetch-url` to read full content from top results
4. Synthesizes best practices from multiple sources
5. Provides structured, comprehensive answer

### Example 3: Documentation Lookup

**User:** "Read the Tokio tutorial and help me build a basic TCP server"

**How it works:**
1. Claude uses `fetch-url` to get tutorial content
2. Analyzes the tutorial structure and examples
3. Adapts examples to your specific needs
4. Provides step-by-step implementation guide
5. Can search for additional information if needed

### Example 4: Comparative Research

**User:** "Compare async runtime performance between Tokio and async-std"

**How it works:**
1. Uses `web-search` to find performance comparisons
2. Uses `web-search` with date filtering for recent benchmarks
3. Uses `fetch-url` to read detailed comparison articles
4. Synthesizes findings from multiple sources
5. Provides balanced comparison

### Example 5: Debugging Help

**User:** "I'm getting a 'future not Send' error with Tokio. Search for solutions and explain them."

**How it works:**
1. Uses `web-search` to find solutions for this error
2. Searches forums, docs, and blog posts
3. Uses `fetch-url` for detailed explanations
4. Provides context-specific solutions
5. Explains why each solution works

## Advanced Examples

### OpenCode: Complex Multi-Step Research

**User:** "Research the latest trends in Rust web frameworks, fetch the documentation for the top 3, and create a comparison table"

**Steps:**
1. `web_search` with `date_range: "month"` for recent articles
2. Identify top frameworks (Axum, Actix, Rocket)
3. `fetch_url` for each framework's documentation
4. Extract key features, performance, and use cases
5. Create detailed comparison table
6. Provide recommendation based on use case

### Claude Code: Iterative Research

**User:** "I want to implement a WebSocket server in Rust. Find examples, review best practices, and help me implement one."

**Steps:**
1. `web-search` for "Rust WebSocket server examples"
2. `fetch-url` for most promising examples
3. `web-search` for "Rust WebSocket best practices"
4. `fetch-url` for authoritative guides
5. Synthesize information
6. Generate implementation with best practices
7. Explain design decisions

### OpenCode: Monitoring Updates

**User:** "Check for any security updates in the Tokio repository from the past day"

**Steps:**
1. `web_search` with:
   - `include_domains: "github.com"`
   - `date_range: "day"`
   - Query: "Tokio security updates"
2. `fetch_url` for release notes or security advisories
3. Summarize any critical updates
4. Provide recommendations for action

### Claude Code: Documentation Creation

**User:** "Help me write documentation for using async/await in my Rust project"

**Steps:**
1. `web-search` for official Rust async documentation
2. `fetch-url` from rust-lang.org async book
3. `web-search` for common patterns and examples
4. `fetch-url` from well-written blog posts
5. Synthesize into project-specific documentation
6. Include examples and best practices

## Tips for Effective Usage

### For OpenCode

1. **Be specific about what you want to search for**
   - Good: "Search for Tokio 1.35 release notes"
   - Less effective: "Tell me about Tokio"

2. **Mention time constraints when relevant**
   - "Find recent articles from this week"
   - "Search for updated documentation from this month"

3. **Specify domains for authoritative sources**
   - "Search rust-lang.org and docs.rs"
   - "Find information from the official Tokio site"

4. **Combine search and fetch**
   - "Find the Tokio tutorial URL and fetch its content"

### For Claude Code

1. **Let Claude decide when to search**
   - Just ask your question naturally
   - Claude will search when it needs current information

2. **Be clear about your goals**
   - "I want to understand X so I can implement Y"
   - "I need to compare X and Y to choose which to use"

3. **Ask for multi-step workflows**
   - "Research X, then help me implement it"
   - "Find examples, explain them, then write similar code"

4. **Request synthesis from multiple sources**
   - "Find multiple perspectives on X and summarize"
   - "Compare what different sources say about Y"

## Troubleshooting

### "No results found"

```bash
# Try a different provider
cli-web-search -p brave "your query"

# Check if any provider is working
cli-web-search providers
```

### "Rate limited"

The tool automatically retries and falls back to other providers. If all fail:

```bash
# Wait a few minutes and try again
# Or configure additional providers for fallback
cli-web-search config set providers.tavily.api_key "your-key"
```

### Skills not loading (Claude Code)

```bash
# Verify skills are loaded
/skills

# Restart with plugin directory
claude --plugin-dir /path/to/cli-web-search
```

## More Resources

- [PLUGINS.md](PLUGINS.md) - Complete plugin documentation
- [QUICK_START_PLUGINS.md](QUICK_START_PLUGINS.md) - Quick reference
- [.opencode/README.md](.opencode/README.md) - OpenCode plugin details
- [.claude-plugin/README.md](.claude-plugin/README.md) - Claude Code plugin details
