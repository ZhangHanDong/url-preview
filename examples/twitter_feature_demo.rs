//! Demonstrates Twitter feature functionality
//!
//! With Twitter feature:
//! cargo run --example twitter_feature_demo --features "twitter"
//!
//! Without Twitter feature:
//! cargo run --example twitter_feature_demo --no-default-features

use std::error::Error;
use url_preview::PreviewService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Twitter Feature Demo ===\n");

    // Feature status
    #[cfg(feature = "twitter")]
    println!("✓ Twitter feature ENABLED - Using oEmbed API");
    #[cfg(not(feature = "twitter"))]
    println!("✗ Twitter feature DISABLED - Using HTML scraping");

    println!();

    // Create preview service
    let service = PreviewService::new();

    // Test with a real Twitter/X URL
    let url = "https://x.com/dotey/status/1899175816711348735";

    println!("Fetching preview for: {}\n", url);

    match service.generate_preview(url).await {
        Ok(preview) => {
            println!("Preview generated successfully:");
            println!("─────────────────────────────");

            if let Some(title) = &preview.title {
                println!("Title:");
                println!("  {}", title);
            }

            if let Some(desc) = &preview.description {
                println!("\nDescription:");
                // Truncate long descriptions for better display
                let display_desc = if desc.len() > 200 {
                    format!("{}...", &desc[..200])
                } else {
                    desc.clone()
                };
                println!("  {}", display_desc);
            }

            if let Some(image) = &preview.image_url {
                println!("\nImage URL:");
                println!("  {}", image);
            }

            if let Some(site) = &preview.site_name {
                println!("\nSite:");
                println!("  {}", site);
            }

            println!("─────────────────────────────");
        }
        Err(e) => {
            println!("Error generating preview: {}", e);
        }
    }

    // Show the difference
    println!("\nKey differences:");
    #[cfg(feature = "twitter")]
    {
        println!("• With Twitter feature:");
        println!("  - Uses Twitter's oEmbed API");
        println!("  - Better formatting of tweet content");
        println!("  - Includes author information");
        println!("  - Properly formatted timestamps");
        println!("  - More reliable content extraction");
    }
    #[cfg(not(feature = "twitter"))]
    {
        println!("• Without Twitter feature:");
        println!("  - Falls back to HTML scraping");
        println!("  - Basic metadata extraction");
        println!("  - May miss some tweet details");
        println!("  - Depends on Twitter's HTML structure");
    }

    // Test URL detection
    println!("\nURL detection test:");
    let test_urls = vec![
        "https://twitter.com/user/status/123",
        "https://x.com/user/status/123",
        "https://github.com/rust-lang/rust",
        "https://example.com",
    ];

    for test_url in test_urls {
        let is_twitter = url_preview::is_twitter_url(test_url);
        println!(
            "  {} -> {}",
            test_url,
            if is_twitter {
                "✓ Twitter"
            } else {
                "✗ Not Twitter"
            }
        );
    }

    Ok(())
}
