//! cli-web-search: A cross-platform CLI web search tool for AI agents

mod cache;
mod cli;
mod config;
mod error;
mod fetch;
mod output;
mod providers;

use cache::SearchCache;
use cli::{CacheCommands, Cli, Commands, ConfigCommands, FetchArgs, FetchFormat};
use config::{config_path, get_config_value, load_config, set_config_value};
use error::{Result, SearchError};
use fetch::{ContentFormat, FetchOptions, Fetcher};
use output::{get_formatter, SearchResponse};
use providers::{build_registry, SearchOptions};
use std::fs;
use std::time::{Duration, Instant};
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
        Commands::Fetch(args) => handle_fetch_command(args).await,
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

async fn handle_fetch_command(args: FetchArgs) -> Result<()> {
    // Convert CLI format to fetch format
    let content_format = match args.format {
        FetchFormat::Text => ContentFormat::Text,
        FetchFormat::Html => ContentFormat::Html,
        FetchFormat::Markdown => ContentFormat::Markdown,
    };

    // Build fetch options
    let options = FetchOptions::new()
        .with_timeout(Duration::from_secs(args.timeout))
        .with_format(content_format)
        .with_max_length(args.max_length);

    let fetcher = Fetcher::with_options(options);

    // Fetch the URL
    if !args.quiet && !args.stdout {
        eprintln!("Fetching: {}", args.url);
    }

    let response = fetcher.fetch(&args.url).await?;

    // Determine output content
    let output_content = if args.json {
        serde_json::to_string_pretty(&response).map_err(|e| SearchError::Api {
            provider: "fetch".to_string(),
            message: format!("Failed to serialize response: {}", e),
        })?
    } else {
        response.content.clone()
    };

    // Handle output
    if args.stdout {
        // Print to stdout
        println!("{}", output_content);
    } else {
        // Determine output file path
        let output_path = if let Some(ref path) = args.output {
            std::path::PathBuf::from(path)
        } else {
            // Generate a temp file path based on URL
            let cache_dir = config::cache_dir()?;
            fs::create_dir_all(&cache_dir)?;

            // Create filename from URL
            let filename = generate_filename_from_url(&args.url, &args.format, args.json);
            cache_dir.join("fetch").join(filename)
        };

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write content to file
        fs::write(&output_path, &output_content)?;

        // Output file location for the agent
        let title_info = response
            .title
            .as_ref()
            .map(|t| format!(" ({})", t))
            .unwrap_or_default();

        if args.json {
            // For JSON output, provide structured response
            println!("{{");
            println!("  \"status\": \"success\",");
            println!("  \"file\": \"{}\",", output_path.display());
            println!("  \"url\": \"{}\",", response.url);
            println!("  \"final_url\": \"{}\",", response.final_url);
            if let Some(title) = &response.title {
                println!("  \"title\": \"{}\",", title.replace('\"', "\\\""));
            }
            println!("  \"content_length\": {},", response.content_length);
            println!(
                "  \"format\": \"{}\"",
                format!("{:?}", args.format).to_lowercase()
            );
            println!("}}");
        } else {
            println!("Fetched: {}{}", response.final_url, title_info);
            println!("Content saved to: {}", output_path.display());
            println!("Size: {} bytes", response.content_length);
        }
    }

    Ok(())
}

/// Generate a filename from a URL
fn generate_filename_from_url(url: &str, format: &FetchFormat, is_json: bool) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Parse URL to get domain and path
    let parsed = url::Url::parse(url).ok();
    let domain = parsed
        .as_ref()
        .and_then(|u| u.host_str())
        .unwrap_or("unknown");

    // Create a sanitized version of the path
    let path_part = parsed
        .as_ref()
        .map(|u| u.path())
        .unwrap_or("")
        .replace('/', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(50)
        .collect::<String>();

    // Add timestamp for uniqueness
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Determine extension
    let ext = if is_json {
        "json"
    } else {
        match format {
            FetchFormat::Html => "html",
            FetchFormat::Markdown => "md",
            FetchFormat::Text => "txt",
        }
    };

    format!("{}{}_{}.{}", domain, path_part, timestamp, ext)
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
