---
name: web-search
description: Search the web for information using cli-web-search. Use when you need current information, research topics, documentation, or want to verify facts. Supports multiple providers (Brave, Google, DuckDuckGo, Tavily, Serper, etc.) with automatic fallback.
---

# Web Search Skill

This skill enables web search capabilities using the cli-web-search command-line tool.

## When to Use This Skill

Use this skill when you need to:
- Find current information or recent events
- Research programming topics, libraries, or frameworks
- Look up documentation or tutorials
- Verify technical facts or best practices
- Discover related resources or solutions
- Get information that's not in your training data

## How to Search

When Claude invokes this skill, it will use the cli-web-search tool with appropriate parameters:

### Basic Search
```bash
cli-web-search -f json "your search query"
```

### Search with Specific Provider
```bash
cli-web-search -f json -p brave "rust async programming"
```

### Filter by Date Range
```bash
cli-web-search -f json --date-range week "latest tokio updates"
```

### Limit Number of Results
```bash
cli-web-search -f json -n 5 "rust error handling"
```

### Domain Filtering
```bash
# Include specific domains
cli-web-search -f json --include-domains "rust-lang.org,docs.rs" "rust documentation"

# Exclude domains
cli-web-search -f json --exclude-domains "stackoverflow.com" "rust tutorials"
```

### Safe Search
```bash
cli-web-search -f json --safe-search strict "programming tutorials"
```

## Search Parameters

- **query**: The search query (required)
- **-p, --provider**: Search provider (brave, google, ddg, tavily, serper, firecrawl, serpapi, bing)
- **-n, --num-results**: Number of results (default: 10)
- **--date-range**: Filter by date (day, week, month, year)
- **--include-domains**: Comma-separated domains to include
- **--exclude-domains**: Comma-separated domains to exclude
- **--safe-search**: Safe search level (off, moderate, strict)
- **-f, --format**: Output format (json for programmatic use)

## Output Format

The tool returns JSON with:
```json
{
  "query": "search query",
  "provider": "brave",
  "search_time_ms": 245,
  "results": [
    {
      "title": "Result Title",
      "url": "https://example.com",
      "snippet": "Description of the result...",
      "position": 1
    }
  ]
}
```

## Best Practices

1. **Be specific**: Use detailed queries for better results
2. **Use recent filters**: For current information, use `--date-range day` or `--date-range week`
3. **Filter domains**: When looking for official docs, use `--include-domains`
4. **Check multiple results**: The first result isn't always the best
5. **Follow up with fetch_url**: After finding relevant URLs, fetch full content for detailed information

## Error Handling

If a search fails:
- Check that cli-web-search is installed: `cli-web-search --version`
- Verify providers are configured: `cli-web-search providers`
- The tool automatically tries fallback providers if one fails
- Check API key configuration if using paid providers

## Examples

### Research a Programming Topic
```bash
cli-web-search -f json -p brave "rust lifetime annotations explained"
```

### Find Recent Documentation
```bash
cli-web-search -f json --date-range month --include-domains "docs.rs,rust-lang.org" "tokio runtime"
```

### Quick Lookup
```bash
cli-web-search -f json -n 3 "rust cargo.toml dependencies"
```

## Related Skills

- **fetch-url**: Fetch full content from specific URLs found in search results
