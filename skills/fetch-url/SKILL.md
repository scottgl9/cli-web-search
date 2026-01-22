---
name: fetch-url
description: Fetch and extract content from a web page using cli-web-search. Use when you need to read documentation, articles, or any web page content. Converts HTML to readable text or markdown format.
---

# URL Fetch Skill

This skill enables fetching and extracting content from web pages using the cli-web-search tool.

## When to Use This Skill

Use this skill when you need to:
- Read documentation from a specific URL
- Extract content from articles or blog posts
- Access web-based resources
- Follow up on search results to get full content
- Download and analyze web page content

## How to Fetch URLs

When Claude invokes this skill, it will use the cli-web-search fetch command:

### Basic Fetch (Text Format)
```bash
cli-web-search fetch "https://example.com" --stdout
```

### Fetch as Markdown
```bash
cli-web-search fetch "https://example.com" -f markdown --stdout
```

### Fetch with Length Limit
```bash
cli-web-search fetch "https://example.com" --max-length 10000 --stdout
```

### Get Metadata as JSON
```bash
cli-web-search fetch "https://example.com" --json
```

## Fetch Parameters

- **url**: The URL to fetch (required)
- **-f, --format**: Output format (text, html, markdown) - default: text
- **--stdout**: Print content to stdout (required for piping to Claude)
- **--max-length**: Maximum content length in bytes (0 = no limit)
- **--json**: Output metadata as JSON instead of content
- **--timeout**: Request timeout in seconds (default: 30)

## Output Formats

### Text Format (Default)
Strips HTML tags and returns plain text:
```
Example Domain

This domain is for use in illustrative examples...
```

### Markdown Format
Converts HTML to markdown, preserving structure:
```markdown
# Example Domain

This domain is for use in illustrative examples...

[More information...](https://www.iana.org/domains/example)
```

### HTML Format
Returns raw HTML:
```html
<!doctype html>
<html>
<head><title>Example Domain</title></head>
...
```

### JSON Metadata
Returns information about the fetch:
```json
{
  "url": "https://example.com",
  "title": "Example Domain",
  "size_bytes": 1256,
  "content_type": "text/html",
  "fetch_time_ms": 123
}
```

## Best Practices

1. **Use markdown format**: For documentation and articles, markdown preserves structure
2. **Set length limits**: For large pages, use `--max-length` to avoid excessive content
3. **Handle errors gracefully**: Some URLs may be inaccessible or rate-limited
4. **Respect robots.txt**: Be mindful of website policies
5. **Combine with search**: First search for relevant URLs, then fetch full content

## Typical Workflow

1. **Search for information**:
   ```bash
   cli-web-search -f json "tokio tutorial"
   ```

2. **Extract relevant URLs** from search results

3. **Fetch full content**:
   ```bash
   cli-web-search fetch "https://tokio.rs/tokio/tutorial" -f markdown --stdout
   ```

4. **Analyze or summarize** the fetched content

## Error Handling

If a fetch fails:
- Check that the URL is accessible in a browser
- Try increasing `--timeout` for slow sites
- Some sites may block automated access
- Check for typos in the URL
- Verify cli-web-search is installed and working

## Examples

### Fetch Documentation Page
```bash
cli-web-search fetch "https://docs.rs/tokio/latest/tokio/" -f markdown --stdout
```

### Get Article Content
```bash
cli-web-search fetch "https://blog.rust-lang.org/2024/01/01/some-post.html" --stdout
```

### Fetch with Size Limit
```bash
cli-web-search fetch "https://github.com/tokio-rs/tokio" --max-length 50000 -f markdown --stdout
```

### Check Page Metadata
```bash
cli-web-search fetch "https://example.com" --json
```

## Related Skills

- **web-search**: Search for URLs before fetching content

## Content Processing

After fetching content, you can:
- Summarize key points
- Extract code examples
- Find specific information
- Compare with other sources
- Create documentation or notes
