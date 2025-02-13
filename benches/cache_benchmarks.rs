use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use url_preview::{Cache, Preview, PreviewService, PreviewServiceConfig};

const MOCK_URLS: &[&str] = &[
    "https://example1.com/page1",
    "https://example2.com/page2",
    "https://example3.com/page3",
    "https://example4.com/page4",
    "https://example5.com/page5",
];

fn create_mock_preview(url: &str) -> Preview {
    Preview {
        url: url.to_string(),
        title: Some(format!("Title for {}", url)),
        description: Some(format!("Description for {}", url)),
        image_url: Some("https://example.com/image.jpg".to_string()),
        favicon: Some("https://example.com/favicon.ico".to_string()),
        site_name: Some("Example Site".to_string()),
    }
}

fn bench_cache_scenarios(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_performance");
    group
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));

    let cache_sizes = [100, 500, 1000];

    for &cache_size in &cache_sizes {
        // No changes needed for no_cache test...
        group.bench_with_input(
            BenchmarkId::new("no_cache", cache_size),
            &cache_size,
            |b, &size| {
                b.iter(|| {
                    let service = PreviewService::with_cache_cap(size);
                    black_box(service)
                });
            },
        );

        // Cache hit test remains the same...
        group.bench_with_input(
            BenchmarkId::new("cache_hit", cache_size),
            &cache_size,
            |b, &_size| {
                let cache = Arc::new(Cache::new(cache_size));

                rt.block_on(async {
                    for url in MOCK_URLS {
                        cache.set(url.to_string(), create_mock_preview(url)).await;
                    }
                });

                b.to_async(&rt).iter(|| async {
                    let url = MOCK_URLS[0];
                    black_box(cache.get(url).await.unwrap())
                });
            },
        );

        // Cache write test with fixed ownership and counter issues
        group.bench_with_input(
            BenchmarkId::new("cache_write", cache_size),
            &cache_size,
            |b, &_size| {
                let cache = Arc::new(Cache::new(cache_size));
                // Use atomic counter for thread-safe counting
                let counter = Arc::new(AtomicUsize::new(0));

                b.to_async(&rt).iter(|| async {
                    let current = counter.fetch_add(1, Ordering::SeqCst);
                    let url = format!("https://dynamic{}.example.com", current);
                    // Clone url before using it
                    let preview = create_mock_preview(&url);
                    black_box(cache.set(url, preview).await)
                });
            },
        );

        // High concurrency test with improved error handling
        group.bench_with_input(
            BenchmarkId::new("concurrent_cache_access", cache_size),
            &cache_size,
            |b, &_size| {
                let cache = Arc::new(Cache::new(cache_size));

                rt.block_on(async {
                    for url in MOCK_URLS {
                        cache.set(url.to_string(), create_mock_preview(url)).await;
                    }
                });

                b.to_async(&rt).iter(|| async {
                    let futures: Vec<_> = MOCK_URLS
                        .iter()
                        .map(|&url| {
                            let cache = Arc::clone(&cache);
                            tokio::spawn(async move {
                                match cache.get(url).await {
                                    Some(preview) => black_box(preview),
                                    None => create_mock_preview(url), // Fallback value
                                }
                            })
                        })
                        .collect();

                    for handle in futures {
                        black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_preview_service_with_cache(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("preview_service_cache");

    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(15))
        .warm_up_time(Duration::from_secs(5));

    let config_with_cache = PreviewServiceConfig {
        cache_capacity: 1000,
        max_concurrent_requests: 10,
        ..Default::default()
    };

    let config_without_cache = PreviewServiceConfig {
        cache_capacity: 0,
        max_concurrent_requests: 10,
        ..Default::default()
    };

    let service_with_cache = Arc::new(PreviewService::new_with_config(config_with_cache));
    let service_without_cache = Arc::new(PreviewService::new_with_config(config_without_cache));

    rt.block_on(async {
        for url in MOCK_URLS {
            let _ = service_with_cache.generate_preview(url).await;
        }
    });

    group.bench_function("service_with_cache", |b| {
        let service = Arc::clone(&service_with_cache);
        b.to_async(&rt).iter(|| async {
            let url = MOCK_URLS[0];
            black_box(service.generate_preview(url).await.unwrap())
        });
    });

    group.bench_function("service_without_cache", |b| {
        let service = Arc::clone(&service_without_cache);
        b.to_async(&rt).iter(|| async {
            let url = MOCK_URLS[0];
            black_box(service.generate_preview(url).await.unwrap())
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(15));
    targets = bench_cache_scenarios, bench_preview_service_with_cache
}
criterion_main!(benches);
