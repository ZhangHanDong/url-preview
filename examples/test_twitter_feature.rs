//! Example to demonstrate the difference between having Twitter feature enabled or disabled
//!
//! Run with Twitter feature:
//! cargo run --example test_twitter_feature --features "twitter"
//!
//! Run without Twitter feature:
//! cargo run --example test_twitter_feature --no-default-features

use std::error::Error;
use url_preview::{CacheStrategy, Fetcher, PreviewGenerator, PreviewService, UrlPreviewGenerator};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Twitter Feature Test ===\n");

    // Check if Twitter feature is enabled
    #[cfg(feature = "twitter")]
    println!("✓ Twitter feature is ENABLED");
    #[cfg(feature = "twitter")]
    println!("  - Using specialized Twitter client with oEmbed API");
    #[cfg(feature = "twitter")]
    println!("  - Better tweet content extraction");
    #[cfg(feature = "twitter")]
    println!("  - Proper author and timestamp handling\n");

    #[cfg(not(feature = "twitter"))]
    println!("✗ Twitter feature is DISABLED");
    #[cfg(not(feature = "twitter"))]
    println!("  - Using standard HTML scraping");
    #[cfg(not(feature = "twitter"))]
    println!("  - Basic metadata extraction only\n");

    // Test URLs - using a real tweet URL for better demonstration
    let test_urls = vec![
        "https://x.com/dotey/status/1899175816711348735", // Real tweet
        "https://twitter.com/rustlang/status/1788218916679229884", // Real tweet
        "https://x.com/github/status/1788589458800443871", // Real tweet
    ];

    println!("Testing URL detection:");
    for url in &test_urls {
        let is_twitter = url_preview::is_twitter_url(url);
        println!("  {} -> is_twitter_url: {}", url, is_twitter);
    }
    println!();

    // Test with PreviewService
    println!("Testing with PreviewService:");
    let service = PreviewService::new();

    for url in &test_urls {
        println!("\nURL: {}", url);
        match service.generate_preview(url).await {
            Ok(preview) => {
                println!("  ✓ Preview generated");
                println!("    Title: {:?}", preview.title);
                println!(
                    "    Description: {:?}",
                    preview.description.map(|d| {
                        if d.len() > 100 {
                            format!("{}...", &d[..100])
                        } else {
                            d
                        }
                    })
                );
                println!("    Site: {:?}", preview.site_name);
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    }

    // Test with direct generator
    println!("\n\nTesting with direct generator:");

    #[cfg(feature = "twitter")]
    {
        println!("Using Twitter-specific client:");
        let twitter_generator = UrlPreviewGenerator::new_with_fetcher(
            0,
            CacheStrategy::NoCache,
            Fetcher::new_twitter_client(),
        );

        let url = "https://x.com/dotey/status/1899175816711348735";
        match twitter_generator.generate_preview(url).await {
            Ok(_preview) => {
                println!("  ✓ Twitter client preview generated");
                println!("    Using oEmbed API");
            }
            Err(e) => {
                println!("  ✗ Twitter client error: {}", e);
            }
        }
    }

    #[cfg(not(feature = "twitter"))]
    {
        println!("Using standard client (Twitter feature not enabled):");
        let standard_generator =
            UrlPreviewGenerator::new_with_fetcher(0, CacheStrategy::NoCache, Fetcher::new());

        let url = "https://x.com/dotey/status/1899175816711348735";
        match standard_generator.generate_preview(url).await {
            Ok(_preview) => {
                println!("  ✓ Standard client preview generated");
                println!("    Using HTML scraping");
            }
            Err(e) => {
                println!("  ✗ Standard client error: {}", e);
            }
        }
    }

    // Show feature compilation info
    println!("\n\nFeature compilation info:");
    println!("  default features: {}", cfg!(feature = "default"));
    println!("  cache feature: {}", cfg!(feature = "cache"));
    println!("  logging feature: {}", cfg!(feature = "logging"));
    println!("  github feature: {}", cfg!(feature = "github"));
    println!("  twitter feature: {}", cfg!(feature = "twitter"));

    // Test fetcher behavior
    println!("\n\nTesting fetcher behavior:");
    let fetcher = Fetcher::new();
    let twitter_url = "https://x.com/dotey/status/1899175816711348735";

    match fetcher.fetch(twitter_url).await {
        Ok(result) => match result {
            url_preview::FetchResult::OEmbed(oembed) => {
                println!("  ✓ Received oEmbed response");
                println!("    Provider: {}", oembed.provider_name);
                println!("    Author: {}", oembed.author_name);
            }
            url_preview::FetchResult::Html(html) => {
                println!("  ✓ Received HTML response");
                println!("    Content length: {} bytes", html.len());
            }
        },
        Err(e) => {
            println!("  ✗ Fetch error: {}", e);
        }
    }

    println!("\n=== Test completed ===");
    Ok(())
}
