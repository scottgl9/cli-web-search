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
}
