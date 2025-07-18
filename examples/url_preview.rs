use url_preview::PreviewService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "logging")]
    log_initialize();

    let preview_service = PreviewService::new();

    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com/zed-industries/zed",
        "https://news.ycombinator.com",
        "https://en.wikipedia.org/wiki/France",
        "https://x.com/dotey/status/1899175816711348735",
    ];

    println!("=== First fetch (from Network):");
    for url in urls.clone() {
        let start = std::time::Instant::now();
        match preview_service.generate_preview(url).await {
            Ok(preview) => {
                let elapsed = start.elapsed();
                println!("URL: {}", url);
                println!("  Title: {:?}", preview.title);
                println!(
                    "  Description: {:?}",
                    preview.description.as_ref().map(|d| &d[..d.len().min(100)])
                );
                println!("  Time taken: {:?}", elapsed);

                #[cfg(feature = "logging")]
                {
                    use url_preview::log_preview_card;
                    log_preview_card(&preview, url);
                    tracing::info!("Title: {:?}", preview.title);
                }
            }
            Err(e) => {
                println!("Error fetching {}: {}", url, e);

                #[cfg(feature = "logging")]
                {
                    use url_preview::log_error_card;
                    log_error_card(url, &e);
                }
            }
        }
        println!();
    }

    println!("\n=== Second fetch (from cache):");
    for url in urls {
        let start = std::time::Instant::now();
        match preview_service.generate_preview(url).await {
            Ok(preview) => {
                let elapsed = start.elapsed();
                println!("URL: {} (cached)", url);
                println!("  Title: {:?}", preview.title);
                println!("  Time taken: {:?}", elapsed);

                #[cfg(feature = "logging")]
                {
                    use url_preview::log_preview_card;
                    log_preview_card(&preview, url);
                    tracing::info!("Time taken for {}: {:?}", url, elapsed);
                }
            }
            Err(e) => {
                println!("Error fetching {}: {}", url, e);

                #[cfg(feature = "logging")]
                {
                    use url_preview::log_error_card;
                    log_error_card(url, &e);
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "logging")]
pub fn log_initialize() {
    use std::path::PathBuf;
    use tracing::info;
    use url_preview::{setup_logging, LogConfig};

    let log_config = LogConfig {
        log_dir: PathBuf::from("logs"),
        log_level: "info".into(),
        console_output: true,
        file_output: true,
    };

    setup_logging(log_config);
    info!("URL Preview system initialized with logging configuration");
}
