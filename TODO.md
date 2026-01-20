# TODO - CLI Web Search Tool

Task tracking for the cli-web-search project. Check items off as completed.

---

## Phase 1: MVP (Target: Week 3)

### Project Setup
- [ ] Initialize Rust project with Cargo
- [ ] Set up project structure (src/, tests/, etc.)
- [ ] Configure Cargo.toml with dependencies
- [ ] Set up CI/CD with GitHub Actions
- [ ] Create initial README.md

### CLI Framework
- [ ] Implement CLI argument parsing with clap
- [ ] Define all command-line options and flags
- [ ] Implement subcommands (config, providers, cache)
- [ ] Add version and help output
- [ ] Implement verbosity levels

### Configuration System
- [ ] Create config file structure (~/.config/cli-web-search/)
- [ ] Implement YAML config parsing with serde
- [ ] Add environment variable support
- [ ] Implement `config init` wizard
- [ ] Implement `config set/get/list` commands
- [ ] Implement `config validate` for API key testing
- [ ] Set proper file permissions (600) for config

### Provider Infrastructure
- [ ] Define SearchProvider trait
- [ ] Create SearchResult struct
- [ ] Implement provider registry
- [ ] Add provider selection logic
- [ ] Implement error handling for provider failures

### Brave Search Provider (P0)
- [ ] Implement Brave Search API client
- [ ] Handle authentication
- [ ] Parse search results
- [ ] Implement rate limit handling
- [ ] Add unit tests
- [ ] Add integration tests

### Google CSE Provider (P0)
- [ ] Implement Google CSE API client
- [ ] Handle authentication (API key + CX)
- [ ] Parse search results
- [ ] Implement rate limit handling
- [ ] Add unit tests
- [ ] Add integration tests

### Output Formatting
- [ ] Define output format trait
- [ ] Implement JSON formatter
- [ ] Implement Markdown formatter
- [ ] Implement plain text formatter
- [ ] Add file output option (-o)

### Error Handling
- [ ] Define custom error types
- [ ] Implement user-friendly error messages
- [ ] Add network error handling
- [ ] Add API error handling
- [ ] Add configuration error handling

---

## Phase 2: Enhanced Features (Target: Week 5)

### Additional Providers
- [ ] Implement DuckDuckGo Instant Answer API
- [ ] Implement Tavily Search API
- [ ] Implement Serper API
- [ ] Add provider status command

### Provider Fallback
- [ ] Implement fallback chain logic
- [ ] Add retry with exponential backoff
- [ ] Handle rate limit detection
- [ ] Add fallback configuration

### Result Caching
- [ ] Design cache storage format
- [ ] Implement cache storage (SQLite or filesystem)
- [ ] Add cache TTL logic
- [ ] Implement cache invalidation
- [ ] Add `cache clear` command
- [ ] Add `cache stats` command
- [ ] Implement --no-cache flag

### Search Filtering
- [ ] Implement --num-results limiting
- [ ] Implement --date-range filtering
- [ ] Implement --include-domains filtering
- [ ] Implement --exclude-domains filtering
- [ ] Implement --safe-search option

---

## Phase 3: Polish (Target: Week 7)

### Documentation
- [ ] Write comprehensive README
- [ ] Add installation instructions
- [ ] Document all CLI options
- [ ] Add provider setup guides
- [ ] Create usage examples
- [ ] Add troubleshooting guide

### Testing
- [ ] Achieve 80%+ code coverage
- [ ] Add end-to-end tests
- [ ] Add mock provider tests
- [ ] Test on Linux x86_64
- [ ] Test on Linux aarch64
- [ ] Test on macOS x86_64
- [ ] Test on macOS aarch64

### CI/CD & Releases
- [ ] Set up GitHub Actions workflow
- [ ] Add automated testing
- [ ] Add linting (clippy)
- [ ] Add formatting check (rustfmt)
- [ ] Configure release builds
- [ ] Create release binaries for all platforms
- [ ] Set up GitHub Releases automation
- [ ] Add checksums for binaries

### Final Polish
- [ ] Performance optimization
- [ ] Binary size optimization
- [ ] Security audit
- [ ] Dependency audit
- [ ] Create demo/screencast

---

## Phase 4: Extended Support (Future)

### Windows Support
- [ ] Test on Windows
- [ ] Handle Windows-specific paths
- [ ] Create Windows installer
- [ ] Update CI for Windows builds

### Additional Providers
- [ ] SerpAPI integration
- [ ] Bing Web Search API
- [ ] Evaluate new search APIs

### Advanced Features
- [ ] Plugin system for custom providers
- [ ] MCP server mode
- [ ] Parallel search across providers
- [ ] Result deduplication
- [ ] Search history

---

## Bug Fixes
<!-- Add bugs as they are discovered -->

---

## Technical Debt
<!-- Track technical debt items -->
- [ ] Review and optimize async code
- [ ] Improve error message clarity
- [ ] Add telemetry (opt-in)

---

## Notes

### Priority Levels
- **P0**: Must have for MVP
- **P1**: Should have, high value
- **P2**: Nice to have
- **P3**: Future consideration

### Dependencies
- clap (CLI parsing)
- reqwest (HTTP client)
- tokio (async runtime)
- serde / serde_json / serde_yaml (serialization)
- thiserror (error handling)
- directories (cross-platform paths)
- colored (terminal colors)
