use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use url_preview::{log_error_card, log_preview_card, setup_logging, LogConfig, PreviewService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_initialize();
    let preview_service = PreviewService::new();

    let urls = vec![
        "https://www.rust-lang.org",
        "https://github.com/zed-industries/zed",
        "https://news.ycombinator.com",
        "https://en.wikipedia.org/wiki/France",
        "https://x.com/dotey/status/1899175816711348735",
    ];

    // https://publish.x.com/oembed?url=https://x.com/dotey/status/1899175816711348735&omit_script=1&lang=en


    info!("=== First fetch (from Network):");
    for url in urls.clone() {
        let start = std::time::Instant::now();
        match preview_service.generate_preview(url).await {
            Ok(preview) => {
                let elapsed = start.elapsed();
                log_preview_card(&preview, url);
                info!("Time taken for {}: {:?}", url, elapsed);
                info!("Title: {:?}", preview.title);
            }
            Err(e) => {
                log_error_card(url, &e);
            }
        }
    }

    info!("=== Second fetch (from cache):");
    for url in urls {
        let start = std::time::Instant::now();
        match preview_service.generate_preview(url).await {
            Ok(preview) => {
                let elapsed = start.elapsed();
                log_preview_card(&preview, url);
                info!("Time taken for {}: {:?}", url, elapsed);
                info!("Title: {:?}", preview.title);
            }
            Err(e) => {
                log_error_card(url, &e);
            }
        }
    }

    Ok(())
}

pub fn log_initialize() {
    let log_config = LogConfig {
        log_dir: PathBuf::from("logs"),
        log_level: "info".into(),
        console_output: true,
        file_output: true,
    };

    setup_logging(log_config);
    info!("URL Preview system initialized with logging configuration");
}
