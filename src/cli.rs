//! CLI argument parsing for cli-web-search

use clap::{Args, Parser, Subcommand, ValueEnum};

/// A cross-platform CLI web search tool for AI agents
#[derive(Parser, Debug)]
#[command(name = "cli-web-search")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The search query
    pub query: Option<String>,

    /// Search provider to use
    #[arg(short, long, value_enum)]
    pub provider: Option<Provider>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Number of results to return
    #[arg(short, long, default_value = "10")]
    pub num_results: usize,

    /// Write output to file
    #[arg(short, long)]
    pub output: Option<String>,

    /// Filter results by date range
    #[arg(long, value_enum)]
    pub date_range: Option<DateRange>,

    /// Only include results from these domains (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub include_domains: Option<Vec<String>>,

    /// Exclude results from these domains (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub exclude_domains: Option<Vec<String>>,

    /// Safe search level
    #[arg(long, value_enum, default_value = "moderate")]
    pub safe_search: SafeSearch,

    /// Bypass result cache
    #[arg(long)]
    pub no_cache: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long)]
    pub quiet: bool,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage configuration
    Config(ConfigArgs),

    /// List available providers and their status
    Providers,

    /// Manage result cache
    Cache(CacheArgs),

    /// Fetch a web page by URL
    Fetch(FetchArgs),
}

/// Configuration subcommand arguments
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Configuration subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Interactive configuration setup
    Init,

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "providers.brave.api_key")
        key: String,
        /// Value to set
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// List all configuration
    List,

    /// Validate API keys
    Validate,

    /// Show configuration file path
    Path,
}

/// Cache subcommand arguments
#[derive(Args, Debug)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

/// Cache subcommands
#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Clear the cache
    Clear,

    /// Show cache statistics
    Stats,
}

/// Fetch subcommand arguments
#[derive(Args, Debug)]
pub struct FetchArgs {
    /// URL to fetch
    pub url: String,

    /// Output format for the fetched content
    #[arg(short, long, value_enum, default_value = "text")]
    pub format: FetchFormat,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Write output to file (default: auto-generated in temp directory)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Maximum content length in bytes (0 = no limit)
    #[arg(long, default_value = "0")]
    pub max_length: usize,

    /// Output as JSON (includes metadata)
    #[arg(long)]
    pub json: bool,

    /// Print content to stdout instead of saving to file
    #[arg(long)]
    pub stdout: bool,

    /// Suppress non-essential output
    #[arg(short, long)]
    pub quiet: bool,
}

/// Fetch output format options
#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum FetchFormat {
    /// Plain text (HTML tags stripped)
    #[default]
    Text,
    /// Raw HTML content
    Html,
    /// Markdown format
    Markdown,
}

/// Available search providers
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum Provider {
    /// Brave Search API
    Brave,
    /// Google Custom Search Engine
    Google,
    /// DuckDuckGo Instant Answer API
    #[value(name = "ddg")]
    DuckDuckGo,
    /// Tavily Search API
    Tavily,
    /// Serper API
    Serper,
    /// Firecrawl Search API
    Firecrawl,
    /// SerpAPI (Google, Bing, Yahoo results)
    #[value(name = "serpapi")]
    SerpApi,
    /// Bing Web Search API
    Bing,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Brave => write!(f, "brave"),
            Provider::Google => write!(f, "google"),
            Provider::DuckDuckGo => write!(f, "duckduckgo"),
            Provider::Tavily => write!(f, "tavily"),
            Provider::Serper => write!(f, "serper"),
            Provider::Firecrawl => write!(f, "firecrawl"),
            Provider::SerpApi => write!(f, "serpapi"),
            Provider::Bing => write!(f, "bing"),
        }
    }
}

/// Output format options
#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON output for programmatic consumption
    Json,
    /// Markdown formatted output
    Markdown,
    /// Plain text output
    #[default]
    Text,
}

/// Date range filter options
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum DateRange {
    /// Past 24 hours
    Day,
    /// Past week
    Week,
    /// Past month
    Month,
    /// Past year
    Year,
}

/// Safe search levels
#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum SafeSearch {
    /// No filtering
    Off,
    /// Moderate filtering
    #[default]
    Moderate,
    /// Strict filtering
    Strict,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_simple_query() {
        let cli = Cli::parse_from(["cli-web-search", "rust programming"]);
        assert_eq!(cli.query, Some("rust programming".to_string()));
        assert_eq!(cli.format, OutputFormat::Text);
        assert_eq!(cli.num_results, 10);
    }

    #[test]
    fn test_cli_parse_with_options() {
        let cli = Cli::parse_from([
            "cli-web-search",
            "-p",
            "brave",
            "-f",
            "json",
            "-n",
            "5",
            "test query",
        ]);
        assert_eq!(cli.provider, Some(Provider::Brave));
        assert_eq!(cli.format, OutputFormat::Json);
        assert_eq!(cli.num_results, 5);
    }

    #[test]
    fn test_cli_parse_config_command() {
        let cli = Cli::parse_from(["cli-web-search", "config", "path"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Config(ConfigArgs {
                command: ConfigCommands::Path
            }))
        ));
    }

    #[test]
    fn test_cli_parse_all_providers() {
        // Test each provider can be parsed
        for (flag, expected) in [
            ("brave", Provider::Brave),
            ("google", Provider::Google),
            ("ddg", Provider::DuckDuckGo),
            ("tavily", Provider::Tavily),
            ("serper", Provider::Serper),
            ("firecrawl", Provider::Firecrawl),
            ("serpapi", Provider::SerpApi),
            ("bing", Provider::Bing),
        ] {
            let cli = Cli::parse_from(["cli-web-search", "-p", flag, "query"]);
            assert_eq!(cli.provider, Some(expected));
        }
    }

    #[test]
    fn test_cli_parse_output_formats() {
        let cli_json = Cli::parse_from(["cli-web-search", "-f", "json", "query"]);
        assert_eq!(cli_json.format, OutputFormat::Json);

        let cli_md = Cli::parse_from(["cli-web-search", "-f", "markdown", "query"]);
        assert_eq!(cli_md.format, OutputFormat::Markdown);

        let cli_text = Cli::parse_from(["cli-web-search", "-f", "text", "query"]);
        assert_eq!(cli_text.format, OutputFormat::Text);
    }

    #[test]
    fn test_cli_parse_date_ranges() {
        for (flag, expected) in [
            ("day", DateRange::Day),
            ("week", DateRange::Week),
            ("month", DateRange::Month),
            ("year", DateRange::Year),
        ] {
            let cli = Cli::parse_from(["cli-web-search", "--date-range", flag, "query"]);
            assert_eq!(cli.date_range, Some(expected));
        }
    }

    #[test]
    fn test_cli_parse_safe_search() {
        for (flag, expected) in [
            ("off", SafeSearch::Off),
            ("moderate", SafeSearch::Moderate),
            ("strict", SafeSearch::Strict),
        ] {
            let cli = Cli::parse_from(["cli-web-search", "--safe-search", flag, "query"]);
            assert_eq!(cli.safe_search, expected);
        }
    }

    #[test]
    fn test_cli_parse_verbosity() {
        let cli_v = Cli::parse_from(["cli-web-search", "-v", "query"]);
        assert_eq!(cli_v.verbose, 1);

        let cli_vv = Cli::parse_from(["cli-web-search", "-vv", "query"]);
        assert_eq!(cli_vv.verbose, 2);

        let cli_vvv = Cli::parse_from(["cli-web-search", "-vvv", "query"]);
        assert_eq!(cli_vvv.verbose, 3);
    }

    #[test]
    fn test_cli_parse_flags() {
        let cli = Cli::parse_from(["cli-web-search", "--no-cache", "--quiet", "query"]);
        assert!(cli.no_cache);
        assert!(cli.quiet);
    }

    #[test]
    fn test_cli_parse_output_file() {
        let cli = Cli::parse_from(["cli-web-search", "-o", "results.json", "query"]);
        assert_eq!(cli.output, Some("results.json".to_string()));
    }

    #[test]
    fn test_cli_parse_timeout() {
        let cli = Cli::parse_from(["cli-web-search", "--timeout", "60", "query"]);
        assert_eq!(cli.timeout, 60);
    }

    #[test]
    fn test_cli_parse_domain_filters() {
        let cli = Cli::parse_from([
            "cli-web-search",
            "--include-domains",
            "rust-lang.org,crates.io",
            "--exclude-domains",
            "spam.com",
            "query",
        ]);
        assert_eq!(
            cli.include_domains,
            Some(vec!["rust-lang.org".to_string(), "crates.io".to_string()])
        );
        assert_eq!(cli.exclude_domains, Some(vec!["spam.com".to_string()]));
    }

    #[test]
    fn test_cli_parse_config_init() {
        let cli = Cli::parse_from(["cli-web-search", "config", "init"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Config(ConfigArgs {
                command: ConfigCommands::Init
            }))
        ));
    }

    #[test]
    fn test_cli_parse_config_set() {
        let cli = Cli::parse_from([
            "cli-web-search",
            "config",
            "set",
            "providers.brave.api_key",
            "key",
        ]);
        match cli.command {
            Some(Commands::Config(ConfigArgs {
                command: ConfigCommands::Set { key, value },
            })) => {
                assert_eq!(key, "providers.brave.api_key");
                assert_eq!(value, "key");
            }
            _ => panic!("Expected Config Set command"),
        }
    }

    #[test]
    fn test_cli_parse_config_get() {
        let cli = Cli::parse_from(["cli-web-search", "config", "get", "default_provider"]);
        match cli.command {
            Some(Commands::Config(ConfigArgs {
                command: ConfigCommands::Get { key },
            })) => {
                assert_eq!(key, "default_provider");
            }
            _ => panic!("Expected Config Get command"),
        }
    }

    #[test]
    fn test_cli_parse_config_list() {
        let cli = Cli::parse_from(["cli-web-search", "config", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Config(ConfigArgs {
                command: ConfigCommands::List
            }))
        ));
    }

    #[test]
    fn test_cli_parse_config_validate() {
        let cli = Cli::parse_from(["cli-web-search", "config", "validate"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Config(ConfigArgs {
                command: ConfigCommands::Validate
            }))
        ));
    }

    #[test]
    fn test_cli_parse_providers_command() {
        let cli = Cli::parse_from(["cli-web-search", "providers"]);
        assert!(matches!(cli.command, Some(Commands::Providers)));
    }

    #[test]
    fn test_cli_parse_cache_clear() {
        let cli = Cli::parse_from(["cli-web-search", "cache", "clear"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Cache(CacheArgs {
                command: CacheCommands::Clear
            }))
        ));
    }

    #[test]
    fn test_cli_parse_cache_stats() {
        let cli = Cli::parse_from(["cli-web-search", "cache", "stats"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Cache(CacheArgs {
                command: CacheCommands::Stats
            }))
        ));
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(format!("{}", Provider::Brave), "brave");
        assert_eq!(format!("{}", Provider::Google), "google");
        assert_eq!(format!("{}", Provider::DuckDuckGo), "duckduckgo");
        assert_eq!(format!("{}", Provider::Tavily), "tavily");
        assert_eq!(format!("{}", Provider::Serper), "serper");
        assert_eq!(format!("{}", Provider::Firecrawl), "firecrawl");
        assert_eq!(format!("{}", Provider::SerpApi), "serpapi");
        assert_eq!(format!("{}", Provider::Bing), "bing");
    }

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert_eq!(format, OutputFormat::Text);
    }

    #[test]
    fn test_safe_search_default() {
        let safe = SafeSearch::default();
        assert_eq!(safe, SafeSearch::Moderate);
    }

    #[test]
    fn test_cli_no_query() {
        let cli = Cli::parse_from(["cli-web-search"]);
        assert!(cli.query.is_none());
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_date_range_equality() {
        assert_eq!(DateRange::Day, DateRange::Day);
        assert_ne!(DateRange::Day, DateRange::Week);
    }

    #[test]
    fn test_cli_parse_fetch_command() {
        let cli = Cli::parse_from(["cli-web-search", "fetch", "https://example.com"]);
        match cli.command {
            Some(Commands::Fetch(args)) => {
                assert_eq!(args.url, "https://example.com");
                assert_eq!(args.format, FetchFormat::Text);
                assert_eq!(args.timeout, 30);
                assert!(!args.json);
                assert!(!args.stdout);
                assert!(!args.quiet);
                assert!(args.output.is_none());
            }
            _ => panic!("Expected Fetch command"),
        }
    }

    #[test]
    fn test_cli_parse_fetch_with_format() {
        for (flag, expected) in [
            ("text", FetchFormat::Text),
            ("html", FetchFormat::Html),
            ("markdown", FetchFormat::Markdown),
        ] {
            let cli =
                Cli::parse_from(["cli-web-search", "fetch", "-f", flag, "https://example.com"]);
            match cli.command {
                Some(Commands::Fetch(args)) => {
                    assert_eq!(args.format, expected);
                }
                _ => panic!("Expected Fetch command"),
            }
        }
    }

    #[test]
    fn test_cli_parse_fetch_with_options() {
        let cli = Cli::parse_from([
            "cli-web-search",
            "fetch",
            "--timeout",
            "60",
            "--max-length",
            "10000",
            "-o",
            "output.txt",
            "https://example.com",
        ]);
        match cli.command {
            Some(Commands::Fetch(args)) => {
                assert_eq!(args.timeout, 60);
                assert_eq!(args.max_length, 10000);
                assert_eq!(args.output, Some("output.txt".to_string()));
            }
            _ => panic!("Expected Fetch command"),
        }
    }

    #[test]
    fn test_cli_parse_fetch_with_flags() {
        let cli = Cli::parse_from([
            "cli-web-search",
            "fetch",
            "--json",
            "--stdout",
            "--quiet",
            "https://example.com",
        ]);
        match cli.command {
            Some(Commands::Fetch(args)) => {
                assert!(args.json);
                assert!(args.stdout);
                assert!(args.quiet);
            }
            _ => panic!("Expected Fetch command"),
        }
    }

    #[test]
    fn test_fetch_format_default() {
        let format = FetchFormat::default();
        assert_eq!(format, FetchFormat::Text);
    }
}
