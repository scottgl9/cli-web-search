# AI Agent Development Guidelines

This document provides guidelines for AI agents (OpenCode, Claude Code, Aider, Cursor, etc.) working on the cli-web-search project.

---

## Project Overview

**cli-web-search** is a cross-platform command-line tool that provides web search capabilities to AI agents and CLI users. It supports multiple search API providers and outputs results in JSON or Markdown format.

### Key Files
- `TODO.md` - Detailed task tracking
- `PROGRESS.md` - Completion progress tracking
- `CLAUDE.md` - Claude-specific development workflow
- `src/` - Rust source code
- `tests/` - Test files
- `Cargo.toml` - Rust project configuration

---

## Development Workflow

### Before Starting Work

1. **Check TODO.md**: See what tasks are pending
2. **Check PROGRESS.md**: Understand current project state
3. **Review existing code**: Understand patterns and conventions

### During Development

1. **Update TODO.md**: Mark tasks as in-progress when starting
2. **Follow Rust conventions**: Use idiomatic Rust patterns
3. **Write tests**: All new functionality should have tests
4. **Handle errors properly**: Use the project's error types
5. **Document public APIs**: Add rustdoc comments

### After Completing Work

1. **Update TODO.md**: Mark completed tasks
2. **Update PROGRESS.md**: Reflect new completion status
3. **Run tests**: Ensure all tests pass (`cargo test`)
4. **Run linter**: Check with clippy (`cargo clippy`)
5. **Format code**: Apply rustfmt (`cargo fmt`)

---

## Code Standards

### Rust Style

```rust
// Use descriptive names
fn search_web(query: &str, provider: &dyn SearchProvider) -> Result<SearchResults, SearchError>

// Document public items
/// Executes a web search using the specified provider.
///
/// # Arguments
/// * `query` - The search query string
/// * `provider` - The search provider implementation
///
/// # Returns
/// Search results or an error if the search failed
pub fn search_web(...) -> Result<...>

// Use proper error handling
match provider.search(query).await {
    Ok(results) => process_results(results),
    Err(SearchError::RateLimited) => try_fallback_provider(query),
    Err(e) => Err(e.into()),
}
```

### Error Handling

- Use `thiserror` for defining error types
- Provide context in error messages
- Never use `.unwrap()` in production code (use `.expect()` sparingly with good messages)
- Propagate errors using `?` operator

### Async Code

- Use `tokio` as the async runtime
- Use `reqwest` for HTTP requests
- Handle timeouts explicitly
- Consider cancellation safety

---

## Project Structure

```
src/
├── main.rs              # Entry point, minimal logic
├── cli.rs               # CLI parsing with clap
├── lib.rs               # Library root (if needed)
├── config/
│   ├── mod.rs           # Config module exports
│   ├── loader.rs        # Load config from files/env
│   └── validation.rs    # Validate configuration
├── providers/
│   ├── mod.rs           # Provider trait + registry
│   ├── brave.rs         # Brave Search
│   ├── google.rs        # Google CSE
│   ├── duckduckgo.rs    # DuckDuckGo
│   ├── tavily.rs        # Tavily
│   ├── serper.rs        # Serper
│   └── firecrawl.rs     # Firecrawl
├── output/
│   ├── mod.rs           # Output formatting trait
│   ├── json.rs          # JSON output
│   ├── markdown.rs      # Markdown output
│   └── text.rs          # Plain text output
├── cache/
│   ├── mod.rs           # Cache interface
│   └── storage.rs       # Cache storage impl
└── error.rs             # Error types
```

### Module Guidelines

- Keep `main.rs` minimal - delegate to library code
- Each provider in its own file implementing `SearchProvider` trait
- Each output format in its own file implementing `OutputFormatter` trait
- Centralize error types in `error.rs`

---

## Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_result() {
        let json = r#"{"title": "Test", "url": "https://example.com"}"#;
        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, "Test");
    }

    #[tokio::test]
    async fn test_provider_search() {
        let provider = MockProvider::new();
        let results = provider.search("test query").await.unwrap();
        assert!(!results.is_empty());
    }
}
```

### Integration Tests

- Place in `tests/` directory
- Use mock servers for API testing
- Test full CLI workflows

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run ignored tests (e.g., tests requiring API keys)
cargo test -- --ignored
```

---

## Common Tasks

### Adding a New Search Provider

1. Create `src/providers/{provider_name}.rs`
2. Implement `SearchProvider` trait
3. Add to provider registry in `src/providers/mod.rs`
4. Add configuration support in `src/config/`
5. Add tests
6. Update documentation

### Adding a New Output Format

1. Create `src/output/{format_name}.rs`
2. Implement `OutputFormatter` trait
3. Register in `src/output/mod.rs`
4. Add CLI option in `src/cli.rs`
5. Add tests

### Adding a New CLI Option

1. Add to clap definition in `src/cli.rs`
2. Handle in main execution logic
3. Update help text
4. Add tests
5. Update documentation

---

## API Key Handling

### Security Requirements

- Never log API keys
- Never include API keys in error messages
- Store config with 600 permissions
- Support environment variable overrides

### Environment Variables

```bash
CLI_WEB_SEARCH_BRAVE_API_KEY=xxx
CLI_WEB_SEARCH_GOOGLE_API_KEY=xxx
CLI_WEB_SEARCH_GOOGLE_CX=xxx
CLI_WEB_SEARCH_TAVILY_API_KEY=xxx
```

---

## Debugging

### Verbose Output

```bash
# Enable verbose logging
cli-web-search -v "search query"

# Extra verbose
cli-web-search -vv "search query"
```

### Environment Variables

```bash
# Enable Rust backtrace
RUST_BACKTRACE=1 cli-web-search "query"

# Debug logging
RUST_LOG=debug cli-web-search "query"
```

---

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy

# Generate docs
cargo doc --open
```

---

## Git Workflow

### Commit Messages

Use conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Maintenance tasks

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates

---

## Questions to Ask

When unclear about implementation details, consider:

1. Is there existing code that handles similar cases?
2. What would provide the best user experience?
3. Is this the simplest solution that works?

---

## Contact

For questions about the project requirements, ask the project maintainer.
