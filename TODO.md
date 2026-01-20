# TODO - CLI Web Search Tool

Task tracking for the cli-web-search project. Check items off as completed.

---

## Phase 1: MVP (Target: Week 3)

### Project Setup
- [x] Initialize Rust project with Cargo
- [x] Set up project structure (src/, tests/, etc.)
- [x] Configure Cargo.toml with dependencies
- [ ] Set up CI/CD with GitHub Actions
- [ ] Create initial README.md

### CLI Framework
- [x] Implement CLI argument parsing with clap
- [x] Define all command-line options and flags
- [x] Implement subcommands (config, providers, cache)
- [x] Add version and help output
- [x] Implement verbosity levels

### Configuration System
- [x] Create config file structure (~/.config/cli-web-search/)
- [x] Implement YAML config parsing with serde
- [x] Add environment variable support
- [x] Implement `config init` wizard
- [x] Implement `config set/get/list` commands
- [x] Implement `config validate` for API key testing
- [x] Set proper file permissions (600) for config

### Provider Infrastructure
- [x] Define SearchProvider trait
- [x] Create SearchResult struct
- [x] Implement provider registry
- [x] Add provider selection logic
- [x] Implement error handling for provider failures

### Brave Search Provider (P0)
- [x] Implement Brave Search API client
- [x] Handle authentication
- [x] Parse search results
- [x] Implement rate limit handling
- [x] Add unit tests
- [ ] Add integration tests

### Google CSE Provider (P0)
- [x] Implement Google CSE API client
- [x] Handle authentication (API key + CX)
- [x] Parse search results
- [x] Implement rate limit handling
- [x] Add unit tests
- [ ] Add integration tests

### Tavily Provider (P1)
- [x] Implement Tavily Search API client
- [x] Handle authentication
- [x] Parse search results
- [x] Implement rate limit handling
- [x] Add unit tests
- [ ] Add integration tests

### Firecrawl Provider (P1)
- [x] Implement Firecrawl Search API client
- [x] Handle authentication
- [x] Parse search results
- [x] Implement rate limit handling
- [x] Add unit tests
- [ ] Add integration tests

### Output Formatting
- [x] Define output format trait
- [x] Implement JSON formatter
- [x] Implement Markdown formatter
- [x] Implement plain text formatter
- [x] Add file output option (-o)

### Error Handling
- [x] Define custom error types
- [x] Implement user-friendly error messages
- [x] Add network error handling
- [x] Add API error handling
- [x] Add configuration error handling

---

## Phase 2: Enhanced Features (Target: Week 5)

### Additional Providers
- [x] Implement DuckDuckGo Instant Answer API
- [x] Implement Serper API
- [x] Implement Firecrawl Search API
- [x] Add provider status command

### Provider Fallback
- [x] Implement fallback chain logic
- [x] Add retry with exponential backoff
- [x] Handle rate limit detection
- [x] Add fallback configuration

### Result Caching
- [x] Design cache storage format
- [x] Implement cache storage (in-memory)
- [x] Add cache TTL logic
- [x] Implement cache invalidation
- [x] Add `cache clear` command
- [x] Add `cache stats` command
- [x] Implement --no-cache flag

### Search Filtering
- [x] Implement --num-results limiting
- [x] Implement --date-range filtering
- [x] Implement --include-domains filtering
- [x] Implement --exclude-domains filtering
- [x] Implement --safe-search option

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
- [x] Test on Linux x86_64
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
- [ ] Persistent cache storage (SQLite or filesystem)

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
- tracing (logging)
