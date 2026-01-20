//! cli-web-search: A cross-platform CLI web search tool for AI agents

mod cache;
mod cli;
mod config;
mod error;
mod output;
mod providers;

use cache::SearchCache;
use cli::{CacheCommands, Cli, Commands, ConfigCommands};
use config::{config_path, get_config_value, load_config, set_config_value};
use error::{Result, SearchError};
use output::{get_formatter, SearchResponse};
use providers::{build_registry, SearchOptions};
use std::fs;
use std::time::Instant;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse_args();

    // Set up logging based on verbosity
    setup_logging(cli.verbose);

    // Handle subcommands first
    if let Some(command) = cli.command {
        return handle_command(command).await;
    }

    // If no query provided, show help
    let query = match &cli.query {
        Some(q) => q.clone(),
        None => {
            eprintln!("Error: No search query provided");
            eprintln!("Usage: cli-web-search <QUERY>");
            eprintln!("       cli-web-search --help");
            std::process::exit(1);
        }
    };

    // Load configuration
    let config = load_config()?;

    // Build provider registry
    let registry = build_registry(&config);

    // Check if any providers are configured
    if registry.configured_providers().is_empty() {
        return Err(SearchError::NoProvidersConfigured);
    }

    // Set up cache
    let cache = SearchCache::new(config.cache.clone());

    // Check cache first (unless disabled)
    let provider_name = cli.provider.as_ref().map(|p| p.to_string());
    if !cli.no_cache {
        if let Some((cached_results, cached_provider)) = cache.get(&query, provider_name.as_deref())
        {
            if !cli.quiet {
                tracing::info!("Using cached results from {}", cached_provider);
            }

            let response = SearchResponse::new(
                query.clone(),
                cached_provider,
                cached_results,
                0, // No search time for cached results
            );

            output_results(&cli, &response)?;
            return Ok(());
        }
    }

    // Build search options
    let options = SearchOptions::new()
        .with_num_results(cli.num_results)
        .with_safe_search(cli.safe_search.clone())
        .with_date_range(cli.date_range.clone())
        .with_timeout(std::time::Duration::from_secs(cli.timeout));

    // Execute search
    let start = Instant::now();
    let (results, provider_used) = registry
        .search_with_fallback(&query, &options, provider_name.as_deref())
        .await?;
    let search_time_ms = start.elapsed().as_millis() as u64;

    // Cache results
    if !cli.no_cache {
        cache.set(&query, provider_used, results.clone());
    }

    // Format and output results
    let response = SearchResponse::new(query, provider_used.to_string(), results, search_time_ms);

    output_results(&cli, &response)?;

    Ok(())
}

fn output_results(cli: &Cli, response: &SearchResponse) -> Result<()> {
    let formatter = get_formatter(&cli.format);
    let output = formatter.format(response);

    // Write to file or stdout
    if let Some(ref path) = cli.output {
        fs::write(path, &output)?;
        if !cli.quiet {
            eprintln!("Results written to {}", path);
        }
    } else {
        println!("{}", output);
    }

    Ok(())
}

async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Config(args) => handle_config_command(args.command).await,
        Commands::Providers => handle_providers_command().await,
        Commands::Cache(args) => handle_cache_command(args.command).await,
    }
}

async fn handle_config_command(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Init => {
            config::init_config_interactive()?;
            let path = config_path()?;
            println!("Configuration initialized at: {}", path.display());
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            set_config_value(&key, &value)?;
            println!("Set {} = {}", key, value);
            Ok(())
        }
        ConfigCommands::Get { key } => {
            match get_config_value(&key)? {
                Some(value) => println!("{}: {}", key, value),
                None => println!("{}: (not set)", key),
            }
            Ok(())
        }
        ConfigCommands::List => {
            let config = load_config()?;
            let map = config.to_flat_map();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                println!("{}: {}", key, map.get(key).unwrap());
            }
            Ok(())
        }
        ConfigCommands::Validate => {
            let config = load_config()?;
            let registry = build_registry(&config);

            println!("Validating API keys...\n");

            for provider in registry.configured_providers() {
                print!("  {}: ", provider.name());
                match provider.validate_api_key().await {
                    Ok(true) => println!("Valid"),
                    Ok(false) => println!("Not configured"),
                    Err(e) => println!("Invalid - {}", e),
                }
            }

            Ok(())
        }
        ConfigCommands::Path => {
            let path = config_path()?;
            println!("{}", path.display());
            Ok(())
        }
    }
}

async fn handle_providers_command() -> Result<()> {
    let config = load_config()?;
    let registry = build_registry(&config);

    println!("Available Search Providers:\n");

    let statuses = registry.list_providers();
    if statuses.is_empty() {
        println!("  No providers registered.");
        println!("\n  Set up a provider with:");
        println!("    cli-web-search config set providers.brave.api_key YOUR_KEY");
    } else {
        for status in &statuses {
            let indicator = if status.configured { "[x]" } else { "[ ]" };
            println!("  {} {}", indicator, status.name);
        }
    }

    // Also show providers that could be configured but aren't registered
    let all_providers = [
        "brave",
        "google",
        "duckduckgo",
        "tavily",
        "serper",
        "firecrawl",
        "serpapi",
        "bing",
    ];
    let registered: Vec<_> = statuses.iter().map(|s| s.name.as_str()).collect();

    let unregistered: Vec<_> = all_providers
        .iter()
        .filter(|p| !registered.contains(*p))
        .collect();

    if !unregistered.is_empty() {
        println!("\n  Not configured:");
        for provider in unregistered {
            println!("  [ ] {}", provider);
        }
    }

    println!("\n  Legend: [x] = configured, [ ] = not configured");

    Ok(())
}

async fn handle_cache_command(command: CacheCommands) -> Result<()> {
    let config = load_config()?;
    let cache = SearchCache::new(config.cache);

    match command {
        CacheCommands::Clear => {
            cache.clear()?;
            println!("Cache cleared.");
            Ok(())
        }
        CacheCommands::Stats => {
            let stats = cache.stats();
            println!("{}", stats);
            Ok(())
        }
    }
}

fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
