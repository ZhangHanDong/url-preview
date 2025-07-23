use clap::{Arg, Command};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use url_preview::{
    CacheStrategy, ContentLimits, Fetcher, FetcherConfig, PreviewError, PreviewService, PreviewServiceConfig,
    UrlValidationConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Secure URL Preview CLI")
        .version("1.0")
        .about("Generate URL previews with security controls")
        .arg(
            Arg::new("urls")
                .help("URLs to preview")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("whitelist")
                .short('w')
                .long("whitelist")
                .help("Only allow these domains (comma-separated)")
                .value_name("DOMAINS"),
        )
        .arg(
            Arg::new("blacklist")
                .short('b')
                .long("blacklist")
                .help("Block these domains (comma-separated)")
                .value_name("DOMAINS"),
        )
        .arg(
            Arg::new("https-only")
                .long("https-only")
                .help("Only allow HTTPS URLs")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("allow-private")
                .long("allow-private")
                .help("Allow private IPs and localhost (DANGEROUS)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max-size")
                .short('s')
                .long("max-size")
                .help("Maximum content size in MB (default: 10)")
                .value_name("SIZE"),
        )
        .arg(
            Arg::new("max-time")
                .short('t')
                .long("max-time")
                .help("Maximum download time in seconds (default: 30)")
                .value_name("SECONDS"),
        )
        .get_matches();

    println!("{}", "Secure URL Preview Generator".bold().green());
    println!("{}", "================================".green());

    // Build security configuration
    let mut url_config = UrlValidationConfig::default();

    // Handle whitelist
    if let Some(whitelist) = matches.get_one::<String>("whitelist") {
        for domain in whitelist.split(',') {
            url_config.allowed_domains.insert(domain.trim().to_string());
        }
        println!(
            "{}: {}",
            "Whitelist mode".yellow(),
            whitelist.replace(',', ", ")
        );
    }

    // Handle blacklist
    if let Some(blacklist) = matches.get_one::<String>("blacklist") {
        for domain in blacklist.split(',') {
            url_config.blocked_domains.insert(domain.trim().to_string());
        }
        println!(
            "{}: {}",
            "Blocked domains".yellow(),
            blacklist.replace(',', ", ")
        );
    }

    // Handle HTTPS-only mode
    if matches.get_flag("https-only") {
        url_config.allowed_schemes.clear();
        url_config.allowed_schemes.insert("https".to_string());
        println!("{}", "HTTPS-only mode enabled".yellow());
    }

    // Handle private IP allowance
    if matches.get_flag("allow-private") {
        url_config.block_private_ips = false;
        url_config.block_localhost = false;
        println!(
            "{}",
            "WARNING: Private IPs and localhost allowed!".red().bold()
        );
    }

    // Build content limits
    let mut content_limits = ContentLimits::default();

    if let Some(max_size) = matches.get_one::<String>("max-size") {
        if let Ok(size_mb) = max_size.parse::<usize>() {
            content_limits.max_content_size = size_mb * 1024 * 1024;
            println!("{}: {} MB", "Max content size".yellow(), size_mb);
        }
    }

    if let Some(max_time) = matches.get_one::<String>("max-time") {
        if let Ok(seconds) = max_time.parse::<u64>() {
            content_limits.max_download_time = seconds;
            println!("{}: {} seconds", "Max download time".yellow(), seconds);
        }
    }

    println!();

    // Create service with security configuration
    let timeout_seconds = content_limits.max_download_time;
    let fetcher_config = FetcherConfig {
        url_validation: url_config,
        content_limits,
        timeout: Duration::from_secs(timeout_seconds),
        ..Default::default()
    };

    let custom_fetcher = Fetcher::with_config(fetcher_config);
    
    let service_config = PreviewServiceConfig::new(1000)
        .with_cache_strategy(CacheStrategy::UseCache)
        .with_default_fetcher(custom_fetcher);

    let service = PreviewService::new_with_config(service_config);

    // Get URLs from command line
    let urls: Vec<&str> = matches
        .get_many::<String>("urls")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    // Create progress bar
    let pb = ProgressBar::new(urls.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut success_count = 0;
    let mut blocked_count = 0;
    let mut error_count = 0;

    let total_urls = urls.len();
    for url in urls {
        match service.generate_preview(url).await {
            Ok(preview) => {
                pb.inc(1);
                success_count += 1;

                println!("\n{}", "✓ URL Preview".bold().green());
                println!("{}", "---------------".green());
                println!("{}: {}", "URL".bold(), url);

                if let Some(title) = preview.title {
                    println!("{}: {}", "Title".bold(), title);
                }

                if let Some(description) = preview.description {
                    println!(
                        "{}: {}",
                        "Description".bold(),
                        if description.len() > 100 {
                            format!("{}...", &description[..100])
                        } else {
                            description
                        }
                    );
                }

                if let Some(image) = preview.image_url {
                    println!("{}: {}", "Image".bold(), image);
                }

                if let Some(site_name) = preview.site_name {
                    println!("{}: {}", "Site Name".bold(), site_name);
                }

                println!();
            }
            Err(e) => {
                pb.inc(1);

                match &e {
                    PreviewError::LocalhostBlocked
                    | PreviewError::PrivateIpBlocked(_)
                    | PreviewError::InvalidUrlScheme(_)
                    | PreviewError::DomainBlocked(_)
                    | PreviewError::DomainNotAllowed(_) => {
                        blocked_count += 1;
                        println!("\n{}: {} - {}", "⚠ Blocked".bold().yellow(), url, e);
                    }
                    PreviewError::ContentSizeExceeded { size, limit } => {
                        error_count += 1;
                        println!(
                            "\n{}: {} - Content too large: {} > {} bytes",
                            "✗ Error".bold().red(),
                            url,
                            size,
                            limit
                        );
                    }
                    PreviewError::DownloadTimeExceeded { elapsed, limit } => {
                        error_count += 1;
                        println!(
                            "\n{}: {} - Download too slow: {}s > {}s",
                            "✗ Error".bold().red(),
                            url,
                            elapsed,
                            limit
                        );
                    }
                    PreviewError::ContentTypeNotAllowed(ct) => {
                        error_count += 1;
                        println!(
                            "\n{}: {} - Content type not allowed: {}",
                            "✗ Error".bold().red(),
                            url,
                            ct
                        );
                    }
                    _ => {
                        error_count += 1;
                        println!("\n{}: {} - {}", "✗ Error".bold().red(), url, e);
                    }
                }
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    pb.finish_with_message("All URLs processed!");

    // Print summary
    println!("\n{}", "Summary".bold().blue());
    println!("{}", "-------".blue());
    println!(
        "{}: {}",
        "Successful".green(),
        success_count.to_string().green()
    );
    println!(
        "{}: {}",
        "Blocked".yellow(),
        blocked_count.to_string().yellow()
    );
    println!("{}: {}", "Errors".red(), error_count.to_string().red());
    println!("{}: {}", "Total", total_urls.to_string().bold());

    Ok(())
}
