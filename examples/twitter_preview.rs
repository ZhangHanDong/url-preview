use std::error::Error;
#[cfg(feature = "logging")]
use std::path::PathBuf;
#[cfg(feature = "logging")]
use tracing::{debug, error, info, warn};
#[cfg(feature = "logging")]
use url_preview::{setup_logging, LogConfig};
use url_preview::{CacheStrategy, Fetcher, PreviewGenerator, UrlPreviewGenerator};

// Macros to handle logging with and without the feature
macro_rules! log_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        info!($($arg)*);
        #[cfg(not(feature = "logging"))]
        println!($($arg)*);
    }};
}

macro_rules! log_warn {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        warn!($($arg)*);
        #[cfg(not(feature = "logging"))]
        eprintln!("WARN: {}", format!($($arg)*));
    }};
}

macro_rules! log_error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        error!($($arg)*);
        #[cfg(not(feature = "logging"))]
        eprintln!("ERROR: {}", format!($($arg)*));
    }};
}

macro_rules! log_debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        debug!($($arg)*);
        #[cfg(not(feature = "logging"))]
        if std::env::var("DEBUG").is_ok() {
            println!("DEBUG: {}", format!($($arg)*));
        }
    }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "logging")]
    {
        let log_config = LogConfig {
            log_dir: PathBuf::from("logs"),
            log_level: "info".into(),
            console_output: true,
            file_output: true,
        };
        setup_logging(log_config);
    }

    log_info!("Starting Twitter/X Preview Example");

    #[cfg(feature = "twitter")]
    let generator = UrlPreviewGenerator::new_with_fetcher(
        1000,
        CacheStrategy::UseCache,
        Fetcher::new_twitter_client(),
    );

    #[cfg(not(feature = "twitter"))]
    let generator =
        UrlPreviewGenerator::new_with_fetcher(1000, CacheStrategy::UseCache, Fetcher::new());

    let url = "https://x.com/blackanger/status/1888945450650362251";
    log_info!("Generating preview for URL: {}", url);

    match generator.generate_preview(url).await {
        Ok(preview) => {
            log_info!("Preview generated successfully");

            log_info!(
                "Preview details:\n  URL: {}\n  Title: {}\n  Description: {}\n  Image: {}\n  Site: {}",
                preview.url,
                preview.title.as_deref().unwrap_or("N/A"),
                preview.description.as_deref().unwrap_or("N/A"),
                preview.image_url.as_deref().unwrap_or("N/A"),
                preview.site_name.as_deref().unwrap_or("N/A")
            );

            if let Some(title) = &preview.title {
                log_debug!("Preview title found: {}", title);
            } else {
                log_warn!("No title found in preview");
            }

            if let Some(desc) = &preview.description {
                log_debug!("Preview description found: {}", desc);
            } else {
                log_warn!("No description found in preview");
            }

            if let Some(image) = &preview.image_url {
                log_debug!("Preview image found: {}", image);
            } else {
                log_warn!("No image found in preview");
            }

            if let Some(site) = &preview.site_name {
                log_debug!("Site name found: {}", site);
            } else {
                log_warn!("No site name found in preview");
            }
        }
        Err(e) => {
            log_error!("Failed to generate preview: {}", e);

            #[cfg(feature = "logging")]
            if let Some(source) = e.source() {
                error!(
                    "Detailed error information - error: {}, source: {}",
                    e, source
                );
            }

            log_warn!("Please check the following:");
            log_warn!("1. URL accessibility");
            log_warn!("2. Network connectivity");
            log_warn!("3. Authentication requirements");
        }
    }

    log_info!("Preview generation process completed");

    #[cfg(feature = "twitter")]
    log_info!("[Twitter feature enabled - using specialized Twitter client]");
    #[cfg(not(feature = "twitter"))]
    log_info!("[Twitter feature disabled - using standard fetcher]");

    Ok(())
}
