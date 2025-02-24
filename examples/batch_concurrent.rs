use std::time::Instant;
use std::{error::Error, path::PathBuf};
use tokio::time::timeout;
use tokio::time::Duration;
use tracing::{info, warn};
use url_preview::{
    setup_logging, FetchResult, Fetcher, FetcherConfig, LogConfig, PreviewService,
    PreviewServiceConfig
};

const BASE_URLS: &[&str] = &[
    "https://www.rust-lang.org",
    "https://github.com/zed-industries/zed",
    "https://news.ycombinator.com",
    "https://www.wikipedia.org",
    "https://github.com/rust-lang/rust",
    "https://github.com/denoland/deno",
    "https://www.reddit.com",
];

struct UrlWithDelay {
    url: String,
    delay: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logging(LogConfig {
        log_dir: PathBuf::from("logs"),
        log_level: "info".into(),
        console_output: true,
        file_output: true,
    });

    info!("Starting batch and concurrent preview example");

    let fetcher_config = FetcherConfig {
        timeout: Duration::from_secs(30),
        user_agent: "batch-test/1.0".into(),
        ..Default::default()
    };

    let test_urls: Vec<UrlWithDelay> = BASE_URLS
        .iter()
        .enumerate()
        .map(|(i, &url)| UrlWithDelay {
            url: url.to_string(),
            delay: (i % 3) as u64,
        })
        .collect();

    let service_config = PreviewServiceConfig {
        cache_capacity: 1000,
        max_concurrent_requests: 3,
        default_fetcher: Some(Fetcher::new_with_config(fetcher_config)),
        ..Default::default()
    };

    let service = PreviewService::new_with_config(service_config);

    info!("\n=== Testing batch processing ===");
    test_improved_batch_processing(&test_urls, &service).await?;

    info!("\n=== Testing concurrency control ===");
    test_improved_concurrent_processing(&test_urls, &service).await?;

    info!("\n=== Performance comparison ===");
    compare_processing_methods(&test_urls).await?;

    Ok(())
}

async fn test_improved_batch_processing(
    urls: &[UrlWithDelay],
    service: &PreviewService,
) -> Result<(), Box<dyn Error>> {
    info!("Starting batch processing with {} URLs", urls.len());
    let start = Instant::now();

    let url_strings: Vec<&str> = urls.iter().map(|u| u.url.as_str()).collect();

    match service
        .default_generator
        .fetcher
        .fetch_batch(url_strings)
        .await
    {
        Ok(results) => {
            let duration = start.elapsed();
            info!("Batch fetch completed in {:?}", duration);

            for (idx, result) in results.iter().enumerate() {
                match result {
                    FetchResult::Html(content) => {
                        info!(
                            "URL: {} - Successfully fetched HTML content ({} bytes)",
                            urls[idx].url,
                            content.len()
                        );
                    }
                    FetchResult::OEmbed(oembed) => {
                        info!(
                            "URL: {} - Successfully fetched oEmbed content from {}",
                            urls[idx].url, oembed.provider_name
                        );
                    }
                }
            }

            let html_count = results
                .iter()
                .filter(|r| matches!(r, FetchResult::Html(_)))
                .count();
            let oembed_count = results
                .iter()
                .filter(|r| matches!(r, FetchResult::OEmbed(_)))
                .count();

            info!("\nBatch Processing Summary:");
            info!("Total URLs processed: {}", results.len());
            info!("HTML responses: {}", html_count);
            info!("oEmbed responses: {}", oembed_count);
            info!(
                "Average time per URL: {:?}",
                duration / results.len() as u32
            );
        }
        Err(e) => {
            warn!("Batch processing failed: {}", e);
            for url_data in urls {
                match service.default_generator.fetcher.fetch(&url_data.url).await {
                    Ok(_) => info!("Individual fetch succeeded for {}", url_data.url),
                    Err(e) => warn!("Individual fetch failed for {}: {}", url_data.url, e),
                }
            }
        }
    }

    Ok(())
}

async fn test_improved_concurrent_processing(
    urls: &[UrlWithDelay],
    service: &PreviewService,
) -> Result<(), Box<dyn Error>> {
    info!("Starting concurrent processing with controlled rate limiting");
    let start = Instant::now();

    let mut handles = vec![];
    for url_data in urls {
        let service = service.clone();
        let url = url_data.url.clone();

        let handle = tokio::spawn(async move {
            match service.generate_preview_with_concurrency(&url).await {
                Ok(preview) => {
                    info!("Concurrent: Successfully processed {}", url);
                    Ok(preview)
                }
                Err(e) => {
                    warn!("Concurrent: Failed to process {}: {}", url, e);
                    Err(e)
                }
            }
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    let duration = start.elapsed();

    let success_count = results
        .iter()
        .filter(|r| r.as_ref().map_or(false, |r| r.is_ok()))
        .count();

    info!(
        "Concurrent processing completed in {:?}. Success: {}/{}",
        duration,
        success_count,
        urls.len()
    );

    Ok(())
}

async fn compare_processing_methods(urls: &[UrlWithDelay]) -> Result<(), Box<dyn Error>> {
    let timeout_duration = Duration::from_secs(20);

    let fetcher_config = FetcherConfig {
        timeout: timeout_duration,
        user_agent: "batch-test/1.0".into(),
        ..Default::default()
    };
    let fetcher = Fetcher::new_with_config(fetcher_config);

    let url_strings: Vec<&str> = urls.iter().map(|u| u.url.as_str()).collect();

    let regular_service = PreviewService::new_with_config(PreviewServiceConfig {
        cache_capacity: 1000,
        default_fetcher: Some(fetcher.clone()),
        ..Default::default()
    });

    let start = Instant::now();
    for url_data in urls {
        match timeout(
            timeout_duration,
            regular_service.generate_preview(&url_data.url),
        )
        .await
        {
            Ok(result) => {
                if let Err(e) = result {
                    warn!("Failed to process {}: {}", url_data.url, e);
                }
            }
            Err(_) => warn!("Request timeout for {}", url_data.url),
        }
    }
    let regular_duration = start.elapsed();
    info!("Regular sequential processing: {:?}", regular_duration);

    let start = Instant::now();
    match timeout(timeout_duration, fetcher.fetch_batch(url_strings.clone())).await {
        Ok(result) => {
            if let Ok(responses) = result {
                info!("Successfully processed {} URLs in batch", responses.len());
            }
        }
        Err(_) => warn!("Batch processing timeout"),
    }
    let batch_duration = start.elapsed();
    info!("Batch processing: {:?}", batch_duration);

    let service_config = PreviewServiceConfig {
        cache_capacity: 1000,
        max_concurrent_requests: 3,
        default_fetcher: Some(fetcher.clone()),
        ..Default::default()
    };

    let concurrent_service = PreviewService::new_with_config(service_config);
    let start = Instant::now();

    let futures: Vec<_> = urls
        .iter()
        .map(|url_data| {
            let service = concurrent_service.clone();
            let url = url_data.url.clone();
            async move {
                timeout(
                    timeout_duration,
                    service.generate_preview_with_concurrency(&url),
                )
                .await
            }
        })
        .collect();

    let results = futures::future::join_all(futures).await;
    let concurrent_duration = start.elapsed();

    let success_count = results
        .iter()
        .filter(|&&ref r| r.as_ref().map_or(false, |r| r.is_ok()))
        .count();

    info!(
        "Concurrent processing: {:?} (Success: {}/{})",
        concurrent_duration,
        success_count,
        urls.len()
    );

    Ok(())
}
