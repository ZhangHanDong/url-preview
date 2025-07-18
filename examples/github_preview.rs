use std::error::Error;
#[cfg(feature = "logging")]
use tracing::info;
use url_preview::PreviewService;

// Macro to handle logging with and without the feature
macro_rules! log_info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "logging")]
        info!($($arg)*);
        #[cfg(not(feature = "logging"))]
        println!($($arg)*);
    }};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "logging")]
    {
        use std::path::PathBuf;
        use url_preview::{setup_logging, LogConfig};

        setup_logging(LogConfig {
            log_dir: PathBuf::from("logs"),
            log_level: "info".into(),
            console_output: true,
            file_output: false,
        });
    }

    log_info!("=== GitHub Preview Example ===\n");

    let service = PreviewService::new();

    // Test various GitHub URLs
    let github_urls = vec![
        "https://github.com/rust-lang/rust",
        "https://github.com/tokio-rs/tokio",
        "https://github.com/ZhangHanDong/url-preview",
        "https://github.com/facebook/react",
        "https://github.com/microsoft/vscode",
    ];

    log_info!("Testing basic preview generation for GitHub repositories:\n");

    for url in &github_urls {
        log_info!("Fetching preview for: {}", url);
        match service.generate_preview(url).await {
            Ok(preview) => {
                log_info!("✓ Success!");
                log_info!("  Title: {:?}", preview.title);
                log_info!("  Description: {:?}", preview.description);
                log_info!("  Image: {:?}", preview.image_url);
                log_info!("  Site: {:?}", preview.site_name);
                log_info!("");
            }
            Err(e) => {
                log_info!("✗ Error: {}\n", e);
            }
        }
    }

    // Test GitHub-specific features if available
    #[cfg(feature = "github")]
    {
        log_info!("\n=== Testing GitHub-specific features ===\n");

        for url in &github_urls[0..2] {
            log_info!("Fetching detailed info for: {}", url);
            match service.get_github_detailed_info(url).await {
                Ok(info) => {
                    log_info!("✓ Detailed info retrieved!");
                    log_info!("  Full name: {}", info.full_name);
                    log_info!("  Description: {}", info.description);
                    log_info!("  Stars: {}", info.stars_count);
                    log_info!("  Forks: {}", info.forks_count);
                    log_info!("  Open issues: {}", info.open_issues_count);
                    log_info!("  Language: {:?}", info.language);
                    log_info!("  Default branch: {}", info.default_branch);
                    log_info!("  Topics: {:?}", info.topics);
                    log_info!("  Homepage: {:?}", info.homepage);
                    log_info!("");
                }
                Err(e) => {
                    log_info!("✗ Error getting detailed info: {}\n", e);
                }
            }
        }
    }
    #[cfg(not(feature = "github"))]
    {
        log_info!("\n[GitHub feature not enabled - detailed info not available]");
    }

    // Test caching
    #[cfg(feature = "cache")]
    {
        log_info!("\n=== Testing cache performance ===\n");

        let test_url = "https://github.com/rust-lang/rust";

        log_info!("First fetch (network):");
        let start = std::time::Instant::now();
        let _ = service.generate_preview(test_url).await?;
        let first_duration = start.elapsed();
        log_info!("  Time: {:?}", first_duration);

        log_info!("Second fetch (cached):");
        let start = std::time::Instant::now();
        let _ = service.generate_preview(test_url).await?;
        let cached_duration = start.elapsed();
        log_info!("  Time: {:?}", cached_duration);

        log_info!(
            "  Speed improvement: {:.1}x faster",
            first_duration.as_secs_f64() / cached_duration.as_secs_f64()
        );
    }

    // Test invalid GitHub URLs
    log_info!("\n=== Testing error handling ===\n");

    let invalid_urls = vec![
        "https://github.com/",
        "https://github.com/nonexistent-user-12345",
        "https://github.com/nonexistent-user-12345/nonexistent-repo-67890",
    ];

    for url in invalid_urls {
        log_info!("Testing invalid URL: {}", url);
        match service.generate_preview(url).await {
            Ok(preview) => {
                log_info!("  Fallback preview generated");
                log_info!("  Title: {:?}", preview.title);
            }
            Err(e) => {
                log_info!("  Error (expected): {}", e);
            }
        }
    }

    log_info!("\n=== Example completed ===");

    Ok(())
}
